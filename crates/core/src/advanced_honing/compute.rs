use ahash::AHashMap;

use crate::advanced_honing::utils::{AdvConfig, AdvDistTriplet, SmallAdvState};

#[derive(Clone, Default, Debug)]
pub struct PMF {
    pub data: Vec<f64>,
}

impl PMF {
    fn new() -> Self {
        Self { data: Vec::new() }
    }

    fn single(val: usize) -> Self {
        let mut data = vec![0.0; val + 1];
        data[val] = 1.0;
        Self { data }
    }

    fn shift_and_scale(&self, shift: usize, scale: f64) -> Self {
        if self.data.is_empty() || scale <= 0.0 {
            return Self::new();
        }
        let mut res = vec![0.0; self.data.len() + shift];
        for (i, &v) in self.data.iter().enumerate() {
            res[i + shift] = v * scale;
        }
        Self { data: res }
    }

    fn add_in_place(&mut self, other: &Self) {
        if self.data.len() < other.data.len() {
            self.data.resize(other.data.len(), 0.0);
        }
        for (i, &v) in other.data.iter().enumerate() {
            self.data[i] += v;
        }
    }
}

const ROLL_THRESH: [(f64, f64); 4] = [
    (0.8, 0.95), // neither
    (0.5, 0.8),  // (juice=True, scroll=False),
    (0.3, 0.75), // (juice=False, scroll=True),
    (0.0, 0.6),  // (juice=True, scroll=True)
];

fn compute_adv_dist(
    state: SmallAdvState,
    config: &AdvConfig,
    memo: &mut AHashMap<SmallAdvState, (PMF, PMF, PMF)>,
) -> (PMF, PMF, PMF) {
    // Terminal condition: if we've reached or exceeded 1000 XP.
    if state.cur_xp >= 100 {
        return (PMF::single(0), PMF::single(0), PMF::single(0));
    }

    // Return memoized distributions if we've visited this state before
    if let Some(dists) = memo.get(&state) {
        return dists.clone();
    }

    let mut expected_cost = PMF::new();
    let mut expected_juice = PMF::new();
    let mut expected_scroll = PMF::new();

    let cost_inc = (!state.next_free) as usize;
    let gracing = state.cur_balls >= 6;
    let double_balls_inc = if config.double_balls { 2 } else { 1 };

    let mut juice_inc = 0;
    let mut scroll_inc = 0;
    let mut is_grace_juice = false;
    let mut is_grace_scroll = false;
    let mut is_non_grace_juice = false;
    let mut is_non_grace_scroll = false;

    let (t1, t2) = {
        let should_juice =
            state.non_grace_juice_count > 0 || (gracing && state.grace_juice_count > 0);
        let should_scroll =
            state.non_grace_scroll_count > 0 || (gracing && state.grace_scroll_count > 0);

        if should_juice {
            juice_inc = 1;
            if gracing {
                is_grace_juice = true;
            } else {
                is_non_grace_juice = true;
            }
        }
        if should_scroll {
            scroll_inc = 1;
            if gracing {
                is_grace_scroll = true;
            } else {
                is_non_grace_scroll = true;
            }
        }

        ROLL_THRESH[(should_juice as usize) | ((should_scroll as usize) << 1)]
    };

    let base_rolls = [(t1, 1), (t2 - t1, 2), (1.0 - t2, 4)];

    for (prob_base, base_xp) in base_rolls {
        if prob_base <= 0.0 {
            continue;
        }

        let mut sub_branches = Vec::new();

        if gracing {
            sub_branches = get_grace_sub_branches(config, &state, base_xp);
        } else {
            let mut next_state = state.clone();
            next_state.cur_balls += double_balls_inc;
            next_state.cur_xp += base_xp;
            next_state.next_free = false; // Next iteration won't be free
            sub_branches.push((1.0, next_state));
        }

        for (prob_sub, mut next_state) in sub_branches {
            // Apply the predefined targets we determined before rolling
            if is_grace_juice {
                next_state.grace_juice_count = if next_state.grace_juice_count == 255 {
                    255
                } else {
                    next_state.grace_juice_count.saturating_sub(1)
                };
            }
            if is_grace_scroll {
                next_state.grace_scroll_count = if next_state.grace_scroll_count == 255 {
                    255
                } else {
                    next_state.grace_scroll_count.saturating_sub(1)
                };
            }
            if is_non_grace_juice {
                next_state.non_grace_juice_count = if next_state.non_grace_juice_count == 255 {
                    255
                } else {
                    next_state.non_grace_juice_count.saturating_sub(1)
                };
            }
            if is_non_grace_scroll {
                next_state.non_grace_scroll_count = if next_state.non_grace_scroll_count == 255 {
                    255
                } else {
                    next_state.non_grace_scroll_count.saturating_sub(1)
                };
            }

            let combined_prob = prob_base * prob_sub;

            // Recurse down the tree
            let (cost_dist, juice_dist, scroll_dist) = compute_adv_dist(next_state, config, memo);

            expected_cost.add_in_place(&cost_dist.shift_and_scale(cost_inc, combined_prob));
            expected_juice.add_in_place(&juice_dist.shift_and_scale(juice_inc, combined_prob));
            expected_scroll.add_in_place(&scroll_dist.shift_and_scale(scroll_inc, combined_prob));
        }
    }

    let result = (expected_cost, expected_juice, expected_scroll);
    memo.insert(state, result.clone());
    result
}

// Helper function to resolve the deterministic branches of "grace" mechanics
fn get_grace_sub_branches(
    config: &AdvConfig,
    state: &SmallAdvState,
    base_xp: u8,
) -> Vec<(f64, SmallAdvState)> {
    let mut branches = Vec::new();
    let is_30_40 = config.is_30_40;

    if state.next_big {
        let probs = [0.2, 0.2, 0.2, 0.2, 0.2];
        for (i, &prob) in probs.iter().enumerate() {
            let mut ns = state.clone();
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
            let mut ns = state.clone();
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
            let mut ns = state.clone();
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
    branches
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

    let result = compute_adv_dist(
        initial_state,
        config,
        &mut AHashMap::new(), //adv_memo_cache.get_mut(&invariant_key).unwrap(),
    );

    AdvDistTriplet {
        cost: result.0.data,
        juice: result.1.data,
        scroll: result.2.data,
    }
}
