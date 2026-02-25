use crate::utils::{Configuration, Output};
use rand::Rng;

const BALL_THRESH_30_40: [f64; 6] = [
    0.125,
    0.25 + 0.125,
    0.125 + 0.25 + 0.125,
    0.25 + 0.125 + 0.25 + 0.125,
    0.125 + 0.25 + 0.125 + 0.25 + 0.125,
    0.125 + 0.125 + 0.25 + 0.125 + 0.25 + 0.125,
];
const BIGBALL_THRESH: [f64; 5] = [0.2, 0.4, 0.6, 0.8, 1.0];

const BALL_THRESH_10_20: [f64; 4] = [0.15, 0.50, 0.65, 1.0];

const ROLL_THRESH: [(f64, f64); 4] = [
    (0.8, 0.95), // neither
    (0.5, 0.8),  // (juice=True, scroll=False),
    (0.3, 0.75), // (juice=False, scroll=True),
    (0.0, 0.6),  // (juice=True, scroll=True)
];

#[inline(always)]
fn big<R: Rng>(
    rng: &mut R,
    next_big: &mut bool,
    rolled_xp: &mut usize,
    cur_xp: &mut usize,
    next_free: &mut bool,
    cur_balls: &mut usize,
) {
    let r = rng.random::<f64>(); // Faster than random_range
    *next_big = false;

    if r < BIGBALL_THRESH[0] {
        *rolled_xp *= 7;
    } else if r < BIGBALL_THRESH[1] {
        *rolled_xp *= 5;
    } else if r < BIGBALL_THRESH[2] {
        *rolled_xp += 80;
        *cur_balls = 6;
    } else if r < BIGBALL_THRESH[3] {
        *rolled_xp += 30;
        *next_free = true;
    } else {
        *cur_xp += *rolled_xp;
        *rolled_xp = 200 - (*cur_xp % 100);
    }
}

#[inline(always)]
fn grace_30_40<R: Rng>(
    rng: &mut R,
    next_big: &mut bool,
    rolled_xp: &mut usize,
    cur_xp: &mut usize,
    next_free: &mut bool,
    cur_balls: &mut usize,
) {
    let r = rng.random::<f64>();
    if r < BALL_THRESH_30_40[0] {
        *rolled_xp *= 5;
    } else if r < BALL_THRESH_30_40[1] {
        *rolled_xp *= 3;
    } else if r < BALL_THRESH_30_40[2] {
        *rolled_xp += 30;
        *cur_balls = 6;
    } else if r < BALL_THRESH_30_40[3] {
        *rolled_xp += 10;
        *next_free = true;
    } else if r < BALL_THRESH_30_40[4] {
        *cur_balls = 6;
        *next_big = true;
    } else {
        *cur_xp += *rolled_xp;
        *rolled_xp = 100 - (*cur_xp % 100);
    }
}

#[inline(always)]
fn grace_10_20<R: Rng>(
    rng: &mut R,
    rolled_xp: &mut usize,
    next_free: &mut bool,
    cur_balls: &mut usize,
) {
    let r = rng.random::<f64>();
    if r < BALL_THRESH_10_20[0] {
        *rolled_xp *= 5;
    } else if r < BALL_THRESH_10_20[1] {
        *rolled_xp *= 3;
    } else if r < BALL_THRESH_10_20[2] {
        *rolled_xp += 30;
        *cur_balls = 6;
    } else {
        *rolled_xp += 10;
        *next_free = true;
    }
}
#[derive(Copy, Clone, Default)]
pub struct Snapshot {
    cost: usize,
    juice: usize,
    scroll: usize,
    cur_balls: usize,
}

pub struct SimTracker {
    history: [Snapshot; 100],
    visited_indices: [usize; 100],
}

impl SimTracker {
    pub fn new() -> Self {
        Self {
            history: [Snapshot::default(); 100],
            visited_indices: [0; 100],
        }
    }
}
pub fn one_sim<R: Rng>(
    rng: &mut R,
    config: &Configuration,
    output: &mut Output,
    tracker: &mut SimTracker,
) {
    let mut cur_xp: usize = 0;
    let mut cur_balls: usize = 0;
    let mut cost: usize = 0;

    let mut non_grace_juice_count: usize = 0;
    let mut non_grace_scroll_count: usize = 0;
    let mut grace_juice_count: usize = 0;
    let mut grace_scroll_count: usize = 0;

    let mut next_free = false;
    let mut next_big = false;

    let is_30_40 = config.is_30_40 == 1;
    let double_balls_inc = if config.double_balls == 1 { 2 } else { 1 };

    // Reset our visit count for this run.
    // We DO NOT need to clear the history array! The count handles the bounds.
    let mut visit_count = 0;

    while cur_xp < 1000 {
        // --- SNAPSHOT LOGIC ---

        // Record the current state before anything increments
        if !next_free {
            let idx = cur_xp / 10;
            if idx < 100 {
                tracker.history[idx] = Snapshot {
                    cost,
                    juice: grace_juice_count + non_grace_juice_count,
                    scroll: grace_scroll_count + non_grace_scroll_count,
                    cur_balls,
                };
                tracker.visited_indices[visit_count] = idx;
                visit_count += 1;
            }
        }
        let gracing = cur_balls >= 6;
        cost += (!next_free) as usize;
        next_free = false;

        let (t1, t2) = if 1000 - cur_xp <= 30 {
            ROLL_THRESH[0]
        } else {
            let should_juice = non_grace_juice_count < config.non_grace_juice_target
                || (gracing && grace_juice_count < config.grace_juice_target);
            let should_scroll = non_grace_scroll_count < config.non_grace_scroll_target
                || (gracing && grace_scroll_count < config.grace_scroll_target);

            if should_juice {
                if gracing {
                    grace_juice_count += 1;
                } else {
                    non_grace_juice_count += 1;
                }
            }
            if should_scroll {
                if gracing {
                    grace_scroll_count += 1;
                } else {
                    non_grace_scroll_count += 1;
                }
            }

            ROLL_THRESH[(should_juice as usize) | ((should_scroll as usize) << 1)]
        };

        let r = rng.random::<f64>();
        let mut rolled_xp: usize = if r < t1 {
            10
        } else if r < t2 {
            20
        } else {
            40
        };

        if gracing {
            cur_balls = 0;
            if next_big {
                big(
                    rng,
                    &mut next_big,
                    &mut rolled_xp,
                    &mut cur_xp,
                    &mut next_free,
                    &mut cur_balls,
                );
            } else if is_30_40 {
                grace_30_40(
                    rng,
                    &mut next_big,
                    &mut rolled_xp,
                    &mut cur_xp,
                    &mut next_free,
                    &mut cur_balls,
                );
            } else {
                grace_10_20(rng, &mut rolled_xp, &mut next_free, &mut cur_balls);
            }
        } else {
            cur_balls += double_balls_inc;
        }
        cur_xp += rolled_xp;
    }

    // --- DISTRIBUTION PROCESSING ---
    // The simulation is complete. Calculate totals.
    let total_juice = grace_juice_count + non_grace_juice_count;
    let total_scroll = grace_scroll_count + non_grace_scroll_count;

    // Work backwards using our visited indices to calculate the delta
    for i in 0..visit_count {
        let idx = tracker.visited_indices[i];
        let snap = tracker.history[idx]; // Copy out the snapshot

        // Final minus Snapshot = Cost to go
        let delta_cost = cost.saturating_sub(snap.cost);
        let delta_juice = total_juice.saturating_sub(snap.juice);
        let delta_scroll = total_scroll.saturating_sub(snap.scroll);
        let balls_back_then = snap.cur_balls;
        // Replace 100 with MAX_COUNT if accessible
        output.cost_dist[balls_back_then][idx][delta_cost] += 1;
        output.juice_dist[balls_back_then][idx][delta_cost] += delta_juice;
        output.scroll_dist[balls_back_then][idx][delta_cost] += delta_scroll;
    }
}
