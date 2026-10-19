use ahash::AHashMap;
use smallvec::SmallVec;

use crate::{
    advanced_honing::utils::{AdvConfig, AdvDistTriplet, SmallAdvState},
    constants::IGNORE_PROB_TOL,
};

/// The cost, juice and scroll distributions of every state visited, end to end in `data`.
/// A state is its index in `spans`: where its three start and how long each is.
struct Dists {
    memo: AHashMap<u64, usize>,
    spans: Vec<(usize, [usize; 3])>,
    data: Vec<f64>,
}

/// the state that is done: nothing more of anything
const DONE: usize = 0;

impl Dists {
    fn new() -> Self {
        Self {
            memo: AHashMap::with_capacity(4096),
            spans: vec![(0, [1, 1, 1])],
            data: vec![1.0; 3],
        }
    }
}

const ROLL_THRESH: [(f64, f64); 4] = [
    (0.8, 0.95), // neither
    (0.5, 0.8),  // (juice=True, scroll=False),
    (0.3, 0.75), // (juice=False, scroll=True),
    (0.0, 0.6),  // (juice=True, scroll=True)
];

// 255 is forever
fn use_one(count: &mut u8) {
    if *count != 255 {
        *count = count.saturating_sub(1);
    }
}

fn compute_adv_dist(state: SmallAdvState, config: &AdvConfig, dists: &mut Dists) -> usize {
    // Terminal condition: if we've reached or exceeded 1000 XP.
    if state.cur_xp >= 100 {
        return DONE;
    }

    // Return memoized distributions if we've visited this state before
    let key = state.packed();
    if let Some(&index) = dists.memo.get(&key) {
        return index;
    }

    let gracing = state.cur_balls >= 6;
    let double_balls_inc = if config.double_balls { 2 } else { 1 };

    let should_juice = (state.non_grace_juice_count > 0 && state.cur_xp <= 96)
        || (gracing && state.grace_juice_count > 0 && state.cur_xp < 94);
    let should_scroll = (state.non_grace_scroll_count > 0 && state.cur_xp <= 96)
        || (gracing && state.grace_scroll_count > 0 && state.cur_xp < 94);
    let (t1, t2) = ROLL_THRESH[(should_juice as usize) | ((should_scroll as usize) << 1)];

    // what this tap adds to the cost, the juice and the scroll
    let incs = [
        (!state.next_free) as usize,
        should_juice as usize,
        should_scroll as usize,
    ];

    let base_rolls = [(t1, 1), (t2 - t1, 2), (1.0 - t2, 4)];
    let mut children: SmallVec<[(f64, usize); 18]> = SmallVec::new();

    for (prob_base, base_xp) in base_rolls {
        if prob_base <= 0.0 {
            continue;
        }

        let mut sub_branches: SmallVec<[(f64, SmallAdvState); 6]> = SmallVec::new();

        if gracing {
            get_grace_sub_branches(config, &state, base_xp, &mut sub_branches);
        } else {
            let mut next_state = state;
            next_state.cur_balls += double_balls_inc;
            next_state.cur_xp += base_xp;
            next_state.next_free = false; // Next iteration won't be free
            sub_branches.push((1.0, next_state));
        }

        for (prob_sub, mut next_state) in sub_branches {
            // Apply the predefined targets we determined before rolling
            if should_juice {
                use_one(if gracing {
                    &mut next_state.grace_juice_count
                } else {
                    &mut next_state.non_grace_juice_count
                });
            }
            if should_scroll {
                use_one(if gracing {
                    &mut next_state.grace_scroll_count
                } else {
                    &mut next_state.non_grace_scroll_count
                });
            }

            let combined_prob = prob_base * prob_sub;
            if combined_prob > IGNORE_PROB_TOL {
                // Recurse down the tree
                children.push((combined_prob, compute_adv_dist(next_state, config, dists)));
            }
        }
    }

    let mut lens = [0; 3];
    for &(_, child) in &children {
        for which in 0..3 {
            lens[which] = lens[which].max(dists.spans[child].1[which] + incs[which]);
        }
    }

    // every child's distributions, shifted by this tap and scaled, straight onto the end
    let start = dists.data.len();
    dists.data.resize(start + lens.iter().sum::<usize>(), 0.0);
    let (known, this) = dists.data.split_at_mut(start);
    for &(prob, child) in &children {
        let (mut from, child_lens) = dists.spans[child];
        let mut to = 0;
        for which in 0..3 {
            let source = &known[from..from + child_lens[which]];
            for (out, &v) in this[to + incs[which]..].iter_mut().zip(source) {
                *out += v * prob;
            }
            from += child_lens[which];
            to += lens[which];
        }
    }

    dists.spans.push((start, lens));
    dists.memo.insert(key, dists.spans.len() - 1);
    dists.spans.len() - 1
}

// Helper function to resolve the deterministic branches of "grace" mechanics
fn get_grace_sub_branches(
    config: &AdvConfig,
    state: &SmallAdvState,
    base_xp: u8,
    branches: &mut SmallVec<[(f64, SmallAdvState); 6]>,
) {
    let is_30_40 = config.is_30_40;

    if state.next_big {
        let probs = [0.2, 0.2, 0.2, 0.2, 0.2];
        for (i, &prob) in probs.iter().enumerate() {
            let mut ns = *state;
            ns.cur_balls = 0;
            ns.next_big = false;
            ns.next_free = false;
            let mut rolled_xp = base_xp;
            let mut cur_xp = state.cur_xp;

            match i {
                0 => rolled_xp *= 7,
                1 => rolled_xp *= 5,
                2 => {
                    rolled_xp += 8;
                    ns.cur_balls = 6;
                }
                3 => {
                    rolled_xp += 3;
                    ns.next_free = true;
                }
                4 => {
                    cur_xp += rolled_xp;
                    rolled_xp = 20 - (cur_xp % 10);
                }
                _ => unreachable!(),
            }
            ns.cur_xp = cur_xp + rolled_xp;
            branches.push((prob, ns));
        }
    } else if is_30_40 {
        let probs = [0.125, 0.25, 0.125, 0.25, 0.125, 0.125];
        for (i, &prob) in probs.iter().enumerate() {
            let mut ns = *state;
            ns.cur_balls = 0;
            ns.next_free = false;
            let mut rolled_xp = base_xp;
            let mut cur_xp = state.cur_xp;

            match i {
                0 => rolled_xp *= 5,
                1 => rolled_xp *= 3,
                2 => {
                    rolled_xp += 3;
                    ns.cur_balls = 6;
                }
                3 => {
                    rolled_xp += 1;
                    ns.next_free = true;
                }
                4 => {
                    ns.cur_balls = 6;
                    ns.next_big = true;
                }
                5 => {
                    cur_xp += rolled_xp;
                    rolled_xp = 10 - (cur_xp % 10);
                }
                _ => unreachable!(),
            }
            ns.cur_xp = cur_xp + rolled_xp;
            branches.push((prob, ns));
        }
    } else {
        let probs = [0.15, 0.35, 0.15, 0.35];
        for (i, &prob) in probs.iter().enumerate() {
            let mut ns = *state;
            ns.cur_balls = 0;
            ns.next_free = false;
            let mut rolled_xp = base_xp;
            let cur_xp = state.cur_xp;

            match i {
                0 => rolled_xp *= 5,
                1 => rolled_xp *= 3,
                2 => {
                    rolled_xp += 3;
                    ns.cur_balls = 6;
                }
                3 => {
                    rolled_xp += 1;
                    ns.next_free = true;
                }
                _ => unreachable!(),
            }
            ns.cur_xp = cur_xp + rolled_xp;
            branches.push((prob, ns));
        }
    }
}

/// Call this function to get your absolute distributions for a given configuration.
pub fn compute_adv_dist_wrapper(config: &AdvConfig) -> AdvDistTriplet {
    let initial_state = SmallAdvState {
        cur_xp: config.start_xp as u8,
        cur_balls: config.start_balls as u8,
        next_free: config.next_free,
        next_big: config.next_big,
        non_grace_juice_count: config.non_grace_juice_target,
        non_grace_scroll_count: config.non_grace_scroll_target,
        grace_juice_count: config.grace_juice_target,
        grace_scroll_count: config.grace_scroll_target,
    };

    let mut dists = Dists::new();
    let index = compute_adv_dist(initial_state, config, &mut dists);
    let (start, lens) = dists.spans[index];
    let juice_start = start + lens[0];
    let scroll_start = juice_start + lens[1];

    AdvDistTriplet {
        cost: dists.data[start..juice_start].to_vec(),
        juice: dists.data[juice_start..scroll_start].to_vec(),
        scroll: dists.data[scroll_start..scroll_start + lens[2]].to_vec(),
    }
}
#[cfg(test)]
mod tests {
    use crate::{
        advanced_honing::utils::{GRACE_FIRST_N, NON_GRACE_FIRST_N, index_to_tuple},
        // helpers::write_jsonl,
    };
    use itertools::iproduct;

    use super::*;

    // Made a start, but here's what needs to happen:
    // 1. figure out a way to embed this data into the binary as binary (not json), via build.rs or otherwise
    // 2. Check in optimize_wrapper if any adv upgrades are of the default settings
    // 3. if so, load state_bundle.adv_cache with this data
    // All this to save like  ~3s? per unique starting position across the whole run
    //
    // ALTERNATIVE
    // embed nothing, pass cache to js on optimizer finish, store in js
    // (maybe also start with the default settings already cached)
    //
    // Either way its pretty marginal gains so cbb right now
    #[test]
    fn compute_cache() {
        let mut grace = GRACE_FIRST_N.to_vec();
        grace.extend_from_slice(&[GRACE_FIRST_N[GRACE_FIRST_N.len() - 1]; NON_GRACE_FIRST_N.len()]);
        let mut non_grace = vec![0; GRACE_FIRST_N.len()];
        non_grace.extend_from_slice(&NON_GRACE_FIRST_N);
        let permitted_strategy_one = grace.into_iter().zip(non_grace.into_iter());
        let all_perms: Vec<((u8, u8), (u8, u8))> = iproduct!(
            permitted_strategy_one
                .clone()
                .into_iter()
                .map(|(x, y)| (x as u8, y as u8)),
            permitted_strategy_one
                .into_iter()
                .map(|(x, y)| (x as u8, y as u8))
        )
        .collect();
        let mut out: Vec<AdvDistTriplet> = Vec::with_capacity(all_perms.len());
        for (
            (grace_juice_target, non_grace_juice_target),
            (grace_scroll_target, non_grace_scroll_target),
        ) in all_perms
        {
            let this_adv_config = AdvConfig {
                start_balls: 0,
                start_xp: 0,
                next_free: false,
                next_big: false,
                double_balls: true,
                is_30_40: false,
                grace_juice_target,
                non_grace_juice_target,
                grace_scroll_target,
                non_grace_scroll_target,
            };
            out.push(compute_adv_dist_wrapper(&this_adv_config));
        }
        // let file_name: String = format!("./adv_cache_data.json",);
        // write_jsonl(&out, &file_name).unwrap();
    }

    /// the distributions are what they were before the copies were taken out, to the bit
    #[test]
    fn dists_are_unchanged() {
        let starts = [
            (0, 0, false, false, false, false),
            (0, 0, false, false, true, false),
            (0, 0, false, false, false, true),
            (37, 3, false, false, false, false),
            (62, 6, false, true, false, true),
            (85, 2, true, false, false, true),
        ];
        let mut acc: u64 = 0;
        for (xp, balls, free, big, double, is_30_40) in starts {
            for (juice, scroll) in iproduct!((0..19).step_by(3), (0..19).step_by(3)) {
                let (juice, scroll) = (index_to_tuple(juice), index_to_tuple(scroll));
                let mut config = AdvConfig::new(xp, balls, free, big, double, is_30_40);
                config.grace_juice_target = juice.0 as u8;
                config.non_grace_juice_target = juice.1 as u8;
                config.grace_scroll_target = scroll.0 as u8;
                config.non_grace_scroll_target = scroll.1 as u8;
                let out = compute_adv_dist_wrapper(&config);
                for dist in [out.cost, out.juice, out.scroll] {
                    acc = acc.wrapping_mul(1099511628211) ^ dist.len() as u64;
                    for p in dist {
                        acc = acc.wrapping_mul(1099511628211) ^ p.to_bits();
                    }
                }
            }
        }
        assert_eq!(acc, 1849612893681050906);
    }
}
