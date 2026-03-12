use crate::advanced_honing::utils::AdvConfig;
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

pub fn one_sim<R: Rng>(rng: &mut R, config: &AdvConfig) -> (u8, u8, u8) {
    let mut cur_xp: usize = 0;
    let mut cur_balls: usize = 0;
    let mut cost: u8 = 0;

    let mut non_grace_juice_count: u8 = 0;
    let mut non_grace_scroll_count: u8 = 0;
    let mut grace_juice_count: u8 = 0;
    let mut grace_scroll_count: u8 = 0;

    let mut next_free = false;
    let mut next_big = false;

    let double_balls_inc = if config.double_balls { 2 } else { 1 };

    while cur_xp < 1000 {
        // --- SNAPSHOT LOGIC ---

        let gracing = cur_balls >= 6;
        cost += (!next_free) as u8;
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
            } else if config.is_30_40 {
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

    let total_juice = grace_juice_count + non_grace_juice_count;
    let total_scroll = grace_scroll_count + non_grace_scroll_count;
    (cost, total_juice, total_scroll)
}
