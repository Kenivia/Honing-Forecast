//! The best plan of one normal upgrade with the rest of the state as it is.
//!
//! The metric is a sum over materials and, the upgrades being independent, an average over
//! the tap this one succeeds on. So what a plan is worth is known for every plan there is
//! from what each amount of each material is worth, and the best plan comes from the last
//! tap backwards. See `AI notes/Calculator core/State space.md`.

use crate::constants::juice_info::JuiceInfo;
use crate::constants::{ARTISAN_MULTIPLIER, FLOAT_TOL, SPECIAL_TOL};
use crate::materials::NUM_BASE_MATS;
use crate::probe::Probe;
use crate::state::{OneState, get_pool_ids};
use crate::state_bundle::StateBundle;
use crate::upgrade::Upgrade;
use smallvec::SmallVec;
use std::cell::RefCell;
use std::sync::atomic::{AtomicUsize, Ordering};

/// One normal upgrade's taps: what a tap may take and what that does to its chance.
/// A plan is the option each tap takes.
#[derive(Clone)]
pub struct Model {
    pub pools: Vec<Vec<usize>>,
    /// every way to take at most one id of each pool, nothing first
    pub options: Vec<OneState>,
    pub extra: Vec<f64>,
    /// what picking the nth id of a pool adds to an option's index
    pub strides: Vec<usize>,
    pub len: usize,
    /// taps that take the same, see `StateBundle::block`
    pub block: usize,
    pub base: f64,
    pub event: f64,
    pub rate: f64,
    pub artisan: f64,
    pub taps: usize,
}

impl Model {
    pub fn of(upgrade: &Upgrade, juice_info: &JuiceInfo, block: usize) -> Model {
        let chance = |id: usize| {
            juice_info
                .access(id, upgrade.piece_type_usize, upgrade.upgrade_index)
                .normal_chance
        };
        let pools: Vec<Vec<usize>> =
            get_pool_ids(upgrade.piece_type_usize, upgrade.upgrade_index, juice_info)
                .into_iter()
                .map(|ids| {
                    ids.into_iter()
                        .filter(|&id| chance(id) > FLOAT_TOL)
                        .collect()
                })
                .collect();
        let mut strides = Vec::new();
        let mut total = 1;
        for ids in &pools {
            strides.push(total);
            total *= ids.len() + 1;
        }
        let options: Vec<OneState> = (0..total)
            .map(|index| {
                pools
                    .iter()
                    .zip(&strides)
                    .filter_map(|(ids, stride)| {
                        (index / stride % (ids.len() + 1))
                            .checked_sub(1)
                            .map(|which| ids[which])
                    })
                    .collect()
            })
            .collect();
        let extra = options
            .iter()
            .map(|option| option.iter().fold(0.0, |sum, &id| sum + chance(id)))
            .collect();
        Model {
            pools,
            options,
            extra,
            strides,
            len: upgrade.state.len(),
            block,
            base: upgrade.base_chance,
            event: upgrade.extra_chance,
            rate: upgrade.artisan_rate,
            artisan: upgrade.starting_artisan,
            taps: upgrade.starting_num_taps,
        }
    }

    pub fn chance(&self, count: usize, extra: f64) -> f64 {
        let ramp = (count + self.taps).min(10) as f64;
        (self.base + self.event + (ramp * self.base) * 0.1 + extra).min(1.0)
    }

    pub fn step(&self, artisan: f64, chance: f64) -> f64 {
        artisan + ARTISAN_MULTIPLIER * chance * self.rate
    }

    /// an id no pool has is dropped
    pub fn indices(&self, plan: &[OneState]) -> Vec<usize> {
        plan.iter()
            .map(|slot| {
                self.pools
                    .iter()
                    .zip(&self.strides)
                    .map(|(ids, stride)| {
                        ids.iter()
                            .position(|id| slot.contains(id))
                            .map_or(0, |which| (which + 1) * stride)
                    })
                    .sum()
            })
            .collect()
    }

    pub fn plan(&self, indices: &[usize]) -> Vec<OneState> {
        indices
            .iter()
            .map(|&option| self.options[option].clone())
            .collect()
    }

    /// the tap that cannot fail, and whether that is a chance of one and not the artisan
    pub fn pity(&self, indices: &[usize]) -> (usize, bool) {
        let mut artisan = self.artisan;
        for count in 0.. {
            if artisan >= 1.0 || count + 1 >= self.len {
                return (count + 1, false);
            }
            let chance = self.chance(count, self.extra[indices[count]]);
            if chance == 1.0 {
                return (count + 1, true);
            }
            artisan = self.step(artisan, chance);
        }
        unreachable!()
    }

    /// The same plan with every tap of a block taking what its first does. A plan cut at its
    /// pity is cut inside a block, and would break the rule there once the pity moved out.
    pub fn filled(&self, indices: &[usize]) -> Vec<usize> {
        (0..indices.len())
            .map(|tap| indices[tap - tap % self.block])
            .collect()
    }

    /// the same plan with nothing on the taps that are never reached or never charged
    pub fn canonical(&self, indices: &[usize]) -> Vec<usize> {
        let (pity, by_chance) = self.pity(indices);
        let live = if by_chance { pity } else { pity - 1 };
        indices
            .iter()
            .enumerate()
            .map(|(tap, &option)| if tap < live { option } else { 0 })
            .collect()
    }
}

pub struct Response {
    pub model: Model,
    /// per pool, where each of its ids is in `juice_worth`
    places: Vec<Vec<usize>>,
    /// by taps paid
    base_worth: Vec<f64>,
    /// by id and times used
    juice_worth: Vec<Vec<f64>>,
    /// whether a plan may take two ids of one pool, on different taps
    mixed: bool,
    /// what the tables took, in evaluations of the metric
    table_work: f64,
    /// what `best` took, in steps of its recursion
    steps: AtomicUsize,
}

/// What `best` has worked out, by tap and times each pick was taken. Most of it is never
/// reached, so it is kept from one response to the next and a cell is of this one only
/// where it carries its stamp.
#[derive(Default)]
struct Memo {
    cells: Vec<(u32, u8, f64)>,
    stamp: u32,
}

thread_local! {
    static MEMO: RefCell<Memo> = RefCell::new(Memo::default());
}

/// What the pools a search leaves alone take on each tap, and what that is worth by taps charged.
struct Kept {
    options: Vec<usize>,
    worth: Vec<f64>,
}

/// The ids a search may take: what taking each adds to an option's index, where it is in
/// `juice_worth`, the most times a plan can take it, and its pool. A tap takes at most one id
/// of a pool.
type Picks = SmallVec<[(usize, usize, usize, usize); 3]>;

/// whether a set of the picks takes two ids of one pool
fn clash(picks: &Picks, set: usize) -> bool {
    (0..picks.len())
        .any(|a| (0..a).any(|b| set >> a & 1 == 1 && set >> b & 1 == 1 && picks[a].3 == picks[b].3))
}

impl StateBundle {
    /// What each of these amounts of one material is worth when this upgrade uses exactly
    /// that, less a constant: the part of the metric of the outcomes it is honed in.
    /// Exact where the lattice answers; elsewhere taken at `Tuning::response_points` of the
    /// amounts and joined by straight lines.
    fn worth(
        &mut self,
        index: usize,
        row: usize,
        amounts: &[f64],
        skips: &[(usize, f64)],
        work: &mut (usize, usize),
        probe: &mut Probe,
    ) -> Vec<f64> {
        let prep = &self.prep_output;
        let bands = prep.bands[prep.optimizer_plan][row].clone();
        let last_price = bands[bands.len() - 1].1;
        let weight: f64 = skips.iter().map(|skip| skip.1).sum();
        if !bands
            .windows(2)
            .any(|pair| (pair[0].1 - pair[1].1).abs() >= FLOAT_TOL)
        {
            return amounts
                .iter()
                .map(|amount| -weight * last_price * amount)
                .collect();
        }
        let kept = self.upgrade_arr[index].cost_dist[row].clone();
        self.upgrade_arr[index].cost_dist[row].pin(0.0);
        work.1 += 1;

        let shifted = |amount: f64| -> SmallVec<[(f64, f64); 4]> {
            bands
                .iter()
                .map(|&(thresh, price)| (thresh - amount, price))
                .collect()
        };
        let thresholds: Vec<f64> = amounts
            .iter()
            .flat_map(|&amount| bands.iter().map(move |pair| pair.0 - amount))
            .collect();
        let counts: SmallVec<[usize; 8]> = skips.iter().map(|skip| skip.0).collect();
        let table = self.lattice_table(row as i64, &counts, &thresholds, probe);
        let sampled: Vec<usize> = {
            let points = self.tuning.response_points.max(2);
            let last = amounts.len() - 1;
            let mut out: Vec<usize> = if amounts.len() <= points {
                (0..=last).collect()
            } else {
                (0..points)
                    .map(|point| point * last / (points - 1))
                    .collect()
            };
            out.dedup();
            out
        };
        let mut out = vec![0.0; amounts.len()];
        for (&(skip, weight), lattice) in skips.iter().zip(&table) {
            match lattice {
                Some(answers) => {
                    for (at, (&amount, answers)) in
                        amounts.iter().zip(answers.chunks(bands.len())).enumerate()
                    {
                        out[at] += weight
                            * self.one_dimension_average_gold(
                                row as i64,
                                skip,
                                &shifted(amount),
                                Some(answers),
                                probe,
                            );
                    }
                }
                None => {
                    work.0 += sampled.len();
                    let values: Vec<f64> = sampled
                        .iter()
                        .map(|&at| {
                            self.one_dimension_average_gold(
                                row as i64,
                                skip,
                                &shifted(amounts[at]),
                                None,
                                probe,
                            )
                        })
                        .collect();
                    let mut pair = 0;
                    for (at, sum) in out.iter_mut().enumerate() {
                        while pair + 2 < sampled.len() && at > sampled[pair + 1] {
                            pair += 1;
                        }
                        let (low, high) = (sampled[pair], sampled[pair + 1]);
                        let share = (at - low) as f64 / (high - low) as f64;
                        *sum += weight * (values[pair] + share * (values[pair + 1] - values[pair]));
                    }
                }
            }
        }
        self.upgrade_arr[index].cost_dist[row] = kept;
        out
    }
}

impl Response {
    pub fn of(state_bundle: &mut StateBundle, index: usize, probe: &mut Probe) -> Response {
        // a rejected move leaves the supports of the state it tried
        state_bundle.update_prob_dist();
        state_bundle.update_cost_dist();
        state_bundle.compute_special_probs(false);

        let this = &state_bundle.upgrade_arr[index];
        let juice_info = &state_bundle.prep_output.juice_info;
        let model = Model::of(this, juice_info, state_bundle.block());
        let ids: Vec<usize> = model.pools.iter().flatten().copied().collect();
        let (start, costs): (Vec<f64>, Vec<f64>) = (0..NUM_BASE_MATS)
            .map(|row| {
                (
                    if this.unlocked {
                        0.0
                    } else {
                        this.unlock_costs[row]
                    },
                    this.costs[row],
                )
            })
            .unzip();
        let each: Vec<(usize, f64)> = ids
            .iter()
            .map(|&id| {
                let amount = juice_info
                    .access(id, this.piece_type_usize, this.upgrade_index)
                    .normal_amt_used as f64;
                (state_bundle.prep_output.table.juice_row(id), amount)
            })
            .collect();
        // the outcomes no free tap skips it in
        let place = state_bundle
            .special_state
            .iter()
            .position(|&u_index| u_index == index)
            .unwrap();
        let skips: Vec<(usize, f64)> = state_bundle
            .special_probs()
            .iter()
            .copied()
            .enumerate()
            .filter(|&(skip, weight)| weight >= SPECIAL_TOL && skip <= place)
            .collect();

        // evaluations of one outcome of one row off the lattice, and rows that needed any
        let mut work = (0, 0);
        let mut base_worth = vec![0.0; model.len + 1];
        for row in (0..NUM_BASE_MATS).filter(|&row| costs[row] > 0.0) {
            let amounts: Vec<f64> = (0..=model.len)
                .map(|taps| start[row] + taps as f64 * costs[row])
                .collect();
            let worth = state_bundle.worth(index, row, &amounts, &skips, &mut work, probe);
            for (sum, worth) in base_worth.iter_mut().zip(&worth) {
                *sum += worth;
            }
        }
        let juice_worth = each
            .iter()
            .map(|&(row, amount)| {
                let amounts: Vec<f64> =
                    (0..=model.len).map(|times| times as f64 * amount).collect();
                // less what none of it is worth, so that an id not taken counts for nothing
                let worth = state_bundle.worth(index, row, &amounts, &skips, &mut work, probe);
                worth.iter().map(|this| this - worth[0]).collect()
            })
            .collect();
        // an evaluation of the metric is every one of those rows under every outcome
        let table_work = work.0 as f64 / (work.1 * skips.len()).max(1) as f64;
        let mut next = 0;
        let places = model
            .pools
            .iter()
            .map(|ids| {
                next += ids.len();
                (next - ids.len()..next).collect()
            })
            .collect();
        let mixed = state_bundle.tuning.response_mixed;
        Response {
            model,
            places,
            base_worth,
            juice_worth,
            mixed,
            table_work,
            steps: AtomicUsize::new(0),
        }
    }

    /// The part of the metric this upgrade's plan moves, less a constant.
    pub fn score(&self, indices: &[usize]) -> f64 {
        let model = &self.model;
        let (mut artisan, mut alive, mut out) = (model.artisan, 1.0, 0.0);
        let mut used = vec![0; self.juice_worth.len()];
        let done = |taps: usize, used: &[usize]| -> f64 {
            self.base_worth[taps]
                + used
                    .iter()
                    .zip(&self.juice_worth)
                    .map(|(&times, worth)| worth[times])
                    .sum::<f64>()
        };
        for count in 0.. {
            if artisan >= 1.0 || count + 1 >= model.len {
                return out + alive * done(count + 1, &used);
            }
            let chance = model.chance(count, model.extra[indices[count]]);
            if chance == 1.0 {
                return out + alive * done(count + 1, &used);
            }
            for (pool, stride) in model.strides.iter().enumerate() {
                if let Some(which) =
                    (indices[count] / stride % (model.pools[pool].len() + 1)).checked_sub(1)
                {
                    used[self.places[pool][which]] += 1;
                }
            }
            out += alive * chance * done(count + 1, &used);
            alive *= 1.0 - chance;
            artisan = model.step(artisan, chance);
        }
        unreachable!()
    }

    /// where a tap and how many times each pick was taken before it are kept
    fn slot(&self, picks: &Picks, count: usize, used: [usize; 3]) -> usize {
        picks
            .iter()
            .zip(used)
            .rev()
            .fold(0, |at, (pick, times)| at * (pick.2 + 1) + times)
            * (self.model.len + 1)
            + count
    }

    /// what succeeding on this tap is worth, the pools left alone having been charged for some taps
    fn done(
        &self,
        picks: &Picks,
        kept: &Kept,
        charged: usize,
        taps: usize,
        used: [usize; 3],
    ) -> f64 {
        self.base_worth[taps]
            + kept.worth[charged]
            + picks
                .iter()
                .zip(used)
                .map(|(pick, times)| self.juice_worth[pick.1][times])
                .sum::<f64>()
    }

    /// what a set of the picks is as an option, and how many times each is then taken
    fn taken(&self, picks: &Picks, set: usize, mut used: [usize; 3]) -> (usize, [usize; 3]) {
        let mut option = 0;
        for (at, pick) in picks.iter().enumerate() {
            if set >> at & 1 == 1 {
                option += pick.0;
                used[at] += 1;
            }
        }
        (option, used)
    }

    /// what a block is worth with this set of the picks on each of its taps, and the best after it
    fn play(
        &self,
        picks: &Picks,
        kept: &Kept,
        start: usize,
        mut artisan: f64,
        mut used: [usize; 3],
        set: usize,
        memo: &mut Memo,
    ) -> f64 {
        let model = &self.model;
        let (mut value, mut alive) = (0.0, 1.0);
        for count in start..start + model.block {
            if artisan >= 1.0 || count + 1 >= model.len {
                return value + alive * self.done(picks, kept, count, count + 1, used);
            }
            let (option, next) = self.taken(picks, set, used);
            let chance = model.chance(count, model.extra[kept.options[count] + option]);
            if chance == 1.0 {
                return value + alive * self.done(picks, kept, count, count + 1, used);
            }
            value += alive * chance * self.done(picks, kept, count + 1, count + 1, next);
            alive *= 1.0 - chance;
            artisan = model.step(artisan, chance);
            used = next;
        }
        value + alive * self.go(picks, kept, start + model.block, artisan, used, memo)
    }

    /// the best from the first tap of a block on
    fn go(
        &self,
        picks: &Picks,
        kept: &Kept,
        count: usize,
        artisan: f64,
        used: [usize; 3],
        memo: &mut Memo,
    ) -> f64 {
        let model = &self.model;
        if artisan >= 1.0 || count + 1 >= model.len {
            return self.done(picks, kept, count, count + 1, used);
        }
        let slot = self.slot(picks, count, used);
        if memo.cells[slot].0 == memo.stamp {
            return memo.cells[slot].2;
        }
        self.steps
            .fetch_add(model.block << picks.len(), Ordering::Relaxed);
        let mut best = (f64::NEG_INFINITY, 0);
        for set in (0..1usize << picks.len()).filter(|&set| !clash(picks, set)) {
            let value = self.play(picks, kept, count, artisan, used, set, memo);
            // a tie goes to the plan that takes nothing
            if set == 0 || value > best.0 + best.0.abs() * 1e-13 {
                best = (value, set as u8);
            }
        }
        memo.cells[slot] = (memo.stamp, best.1, best.0);
        best.0
    }

    /// The best of the plans that take these ids and leave the other pools as kept, every
    /// block filled.
    fn best_of(&self, picks: &Picks, kept: &Kept) -> (f64, Vec<usize>) {
        let model = &self.model;
        let size = picks.iter().map(|pick| pick.2 + 1).product::<usize>() * (model.len + 1);
        let mut memo = MEMO.take();
        if memo.cells.len() < size {
            memo.cells.resize(size, (0, 0, 0.0));
        }
        memo.stamp = memo.stamp.wrapping_add(1);
        if memo.stamp == 0 {
            memo.cells.fill((0, 0, 0.0));
            memo.stamp = 1;
        }
        let value = self.go(picks, kept, 0, model.artisan, [0; 3], &mut memo);
        let mut indices = kept.options.clone();
        let (mut artisan, mut used) = (model.artisan, [0; 3]);
        let mut set = 0;
        for count in 0.. {
            if artisan >= 1.0 || count + 1 >= model.len {
                break;
            }
            if count % model.block == 0 {
                // two orders of the same taps can differ in the last bit of the artisan, and one of them stop a tap early
                let (stamp, chosen, _) = memo.cells[self.slot(picks, count, used)];
                if stamp != memo.stamp {
                    break;
                }
                set = chosen;
            }
            let (option, next) = self.taken(picks, set as usize, used);
            indices[count] += option;
            let chance = model.chance(count, model.extra[indices[count]]);
            if chance == 1.0 {
                break;
            }
            used = next;
            artisan = model.step(artisan, chance);
        }
        MEMO.set(memo);
        (value, model.filled(&indices))
    }

    /// What this response has cost so far, in evaluations of the metric.
    pub fn work(&self, steps_per_evaluation: f64) -> f64 {
        self.table_work + self.steps.load(Ordering::Relaxed) as f64 / steps_per_evaluation
    }

    fn pick(&self, pool: usize, which: usize) -> (usize, usize, usize, usize) {
        let option = self.model.strides[pool] * (which + 1);
        // taken on every tap it is taken the most times
        (
            option,
            self.places[pool][which],
            self.model.pity(&vec![option; self.model.len]).0,
            pool,
        )
    }

    /// a plan less what some pools take, and what the rest of it is worth by taps charged
    fn kept(&self, indices: &[usize], without: &[usize]) -> Kept {
        let model = &self.model;
        let mut options = indices.to_vec();
        let mut used = vec![0; self.juice_worth.len()];
        let mut worth = vec![0.0; model.len + 1];
        for tap in 0..model.len {
            for (pool, stride) in model.strides.iter().enumerate() {
                let Some(which) =
                    (indices[tap] / stride % (model.pools[pool].len() + 1)).checked_sub(1)
                else {
                    continue;
                };
                if without.contains(&pool) {
                    options[tap] -= stride * (which + 1);
                } else {
                    used[self.places[pool][which]] += 1;
                }
            }
            worth[tap + 1] = used
                .iter()
                .zip(&self.juice_worth)
                .map(|(&times, worth)| worth[times])
                .sum();
        }
        Kept { options, worth }
    }

    /// `best` where that is at most this many steps, and `improve` from the plan where not.
    pub fn better(&self, indices: &[usize], joint_steps: usize) -> (f64, Vec<usize>) {
        let model = &self.model;
        let (mut cells, mut choices) = (model.len + 1, 1);
        for (pool, ids) in model
            .pools
            .iter()
            .enumerate()
            .filter(|(_, ids)| !ids.is_empty())
        {
            cells *= 2 * (self.pick(pool, 0).2 + 1);
            choices *= ids.len();
        }
        // a third of the cells are reached, about
        if cells / 3 * choices > joint_steps {
            return self.improve(indices);
        }
        // the best with one id a pool, and from it with two where a pool has them
        let found = self.best();
        if choices == 1 || !self.mixed {
            return found;
        }
        self.improve(if self.score(indices) > found.0 {
            indices
        } else {
            &found.1
        })
    }

    /// The best of every plan that keeps to one id a pool. Two ids of a pool in one plan
    /// are left out: it was the best of a lone upgrade once in thirty, by 0.02% of flow.
    pub fn best(&self) -> (f64, Vec<usize>) {
        let model = &self.model;
        let used_pools: Vec<usize> = (0..model.pools.len())
            .filter(|&pool| !model.pools[pool].is_empty())
            .collect();
        let kept = self.kept(&vec![0; model.len], &[]);
        let mut which = vec![0; used_pools.len()];
        let mut best = (f64::NEG_INFINITY, vec![0; model.len]);
        loop {
            let mut picks: Picks = used_pools
                .iter()
                .zip(&which)
                .map(|(&pool, &which)| self.pick(pool, which))
                .collect();
            // three long pools would not fit; the last one goes unused
            while picks.iter().map(|pick| pick.2 + 1).product::<usize>() * (model.len + 1)
                > 2_000_000
            {
                picks.pop();
            }
            let found = self.best_of(&picks, &kept);
            if found.0 > best.0 {
                best = (found.0, model.canonical(&found.1));
            }
            // the next choice of ids
            let Some(at) =
                (0..which.len()).find(|&at| which[at] + 1 < model.pools[used_pools[at]].len())
            else {
                return best;
            };
            which[at] += 1;
            which[..at].fill(0);
        }
    }

    /// From a plan, each pool in turn takes its best taps with the others as they are, until
    /// none moves. A plan no single pool can better, at a fraction of the work of `best`
    /// where there are two pools.
    pub fn improve(&self, indices: &[usize]) -> (f64, Vec<usize>) {
        let model = &self.model;
        let mut now = (self.score(indices), model.filled(&model.canonical(indices)));
        let used_pools: Vec<usize> = (0..model.pools.len())
            .filter(|&pool| !model.pools[pool].is_empty())
            .collect();
        for _ in 0..4 {
            let mut moved = false;
            for &pool in &used_pools {
                let kept = self.kept(&now.1, &[pool]);
                let ids = model.pools[pool].len();
                // one id of the pool, then two of them, a tap taking either
                let alone = (0..ids)
                    .map(|which| std::iter::once(self.pick(pool, which)).collect::<Picks>());
                let pairs = (0..ids)
                    .flat_map(|a| (0..a).map(move |b| (a, b)))
                    .filter(|_| self.mixed)
                    .map(|(a, b)| {
                        [self.pick(pool, b), self.pick(pool, a)]
                            .into_iter()
                            .collect::<Picks>()
                    });
                for picks in alone.chain(pairs) {
                    if picks.iter().map(|pick| pick.2 + 1).product::<usize>() * (model.len + 1)
                        > 2_000_000
                    {
                        continue;
                    }
                    let found = self.best_of(&picks, &kept);
                    if found.0 > now.0 + now.0.abs() * 1e-12 {
                        now = found;
                        moved = true;
                    }
                }
            }
            if !moved || used_pools.len() < 2 && model.pools[used_pools[0]].len() < 2 {
                break;
            }
        }
        (now.0, model.canonical(&now.1))
    }
}
