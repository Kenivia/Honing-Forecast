use crate::constants::FLOAT_TOL;
use crate::performance::Performance;

pub static THETA_TOL: f64 = 1e-10;
pub static THETA_LIMIT: f64 = 1e2; // th
pub fn ks_01234(
    log_prob_dist_arr: &[Vec<f64>],
    support_arr: &[Vec<f64>],
    theta: f64,
    (total_k, total_k1, total_k2, total_k3, total_k4): &mut (f64, f64, f64, f64, f64),
    toggle: &(bool, bool, bool, bool, bool), // honestly the performance gain here is probably so negligible compared to all the exp and ln calls but whatever
) {
    for (u_index, log_prob_dist) in log_prob_dist_arr.iter().enumerate() {
        let support = &support_arr[u_index];
        let mut ignore: bool = true;
        for i in support {
            if *i > 0.0 {
                ignore = false;
                break;
            }
        }
        if ignore {
            continue;
        }

        let mut alpha_arr: Vec<f64> = Vec::with_capacity(log_prob_dist.len());
        let mut alpha_shift: f64 = f64::NEG_INFINITY;

        let mut sanity_check: f64 = 0.0;
        for (p_index, l) in log_prob_dist.iter().enumerate() {
            let this_alpha: f64 = l + theta * support[p_index];

            alpha_arr.push(this_alpha);
            alpha_shift = this_alpha.max(alpha_shift);
            sanity_check += l.exp();
        }
        if (1.0 - sanity_check).abs() > FLOAT_TOL {
            dbg!(
                sanity_check,
                log_prob_dist,
                log_prob_dist_arr
                    .iter()
                    .map(|x| x.iter().map(|y| y.exp()).sum::<f64>())
                    .collect::<Vec<f64>>(),
                log_prob_dist_arr
                    .iter()
                    .map(|x| x.iter().map(|y| y.exp()).collect())
                    .collect::<Vec<Vec<f64>>>(),
            );
            panic!();
        }

        let mut s: f64 = 0.0;
        let mut mean: f64 = 0.0;
        let mut second: f64 = 0.0;
        let mut third: f64 = 0.0;
        let mut fourth: f64 = 0.0;

        // if theta == 0.0 && DEBUG {
        //     dbg!(&alpha_arr);
        // }

        let mut u_arr: Vec<f64> = Vec::with_capacity(log_prob_dist.len());
        for aj in alpha_arr.iter() {
            if *aj == f64::NEG_INFINITY {
                // just to make it explicit, i think exp does this anyway
                u_arr.push(0.0);
                continue;
            }
            let u: f64 = (aj - alpha_shift).exp(); // i dont think this can be turned into a poly? cos cur_gold_cost is not linear (maybe do this for special?)
            s += u;
            u_arr.push(u);
        }

        for (p_index, &u) in u_arr.iter().enumerate() {
            if u == 0.0 {
                //   l = -inf , p = 0
                continue;
            }
            let w = u / s;
            let x = support[p_index];

            if toggle.1 || toggle.2 || toggle.3 || toggle.4 {
                mean += x * w;
                if toggle.2 || toggle.3 || toggle.4 {
                    let x2: f64 = x * x;
                    second += x2 * w;
                    if toggle.3 || toggle.4 {
                        let x3: f64 = x2 * x;
                        third += x3 * w;
                        if toggle.4 {
                            fourth += x3 * x * w;
                        }
                    }
                }
            }
        }

        if toggle.0 {
            *total_k += alpha_shift + s.ln();
        }
        let mut mu2: f64 = -1.0;
        if toggle.1 {
            *total_k1 += mean;
        }
        if toggle.2 || toggle.4 {
            mu2 = (second - mean * mean).max(0.0);
            *total_k2 += mu2;
        }
        if toggle.3 {
            let mu3 = third - 3.0 * second * mean + 2.0 * mean * mean * mean;
            *total_k3 += mu3;
        }
        if toggle.4 {
            if !toggle.2 {
                mu2 = (second - mean * mean).max(0.0);
            }
            let mu4 = fourth - 4.0 * third * mean + 6.0 * second * mean * mean
                - 3.0 * mean * mean * mean * mean;
            *total_k4 += mu4 - 3.0 * mu2 * mu2;
        }
    }
}

pub fn find_root<F>(
    mut func: F,
    init_theta: f64,
    min: f64,
    max: f64,
    tol: f64,
    max_iter: usize,
    performance: &mut Performance,
) -> Option<(f64, f64)>
where
    F: FnMut(f64) -> (f64, f64),
{
    let mut theta: f64 = init_theta.max(min).min(max);

    let mut count = 0;
    performance.ks_count += 1;
    let (mut y, mut dy) = func(theta);
    loop {
        // dbg!(count, theta, y, dy);
        // if y.abs() < 1e-12 {
        //     // this is largely irrelevant because we're interested in theta
        //     return Some(theta);
        // }

        if dy == 0.0 {
            // let (y_min, _) = func(min);
            // let (y_max, _) = func(max);
            dbg!(y, dy, theta);
            // this shouldn't happen no more
            return None;
            // return Some((
            //     if y_min.abs() < y_max.abs() { min } else { max },
            //     if y_min.abs() < y_max.abs() {
            //         min + THETA_TOL
            //     } else {
            //         max - THETA_TOL
            //     },
            // ));
        }

        let proposed_delta: f64 = -y / dy;
        let mut cur_delta: f64 = proposed_delta;
        let mut damping_factor = 1.0;
        let mut new_theta: f64 = theta + cur_delta;
        let (mut new_y, mut new_dy);
        loop {
            performance.ks_count += 1;
            (new_y, new_dy) = func(new_theta);

            // If the error (magnitude of y) got worse, reduce step size
            if new_y.abs() > y.abs() || new_dy == 0.0 {
                damping_factor *= 0.5;
                cur_delta = proposed_delta * damping_factor;
                new_theta = theta + cur_delta;

                if damping_factor < 1e-3 {
                    break;
                }
            } else {
                // The step is good, it reduced the error
                break;
            }
        }
        if !new_theta.is_finite() {
            dbg!(y, dy, proposed_delta, theta);
            return None;
        }

        // assert!(new_theta < max && new_theta > min);

        count += 1;
        performance.newton_iterations += 1;
        if (new_theta - theta).abs() < tol || count >= max_iter {
            return Some((new_theta, theta));
        }

        theta = new_theta;
        (y, dy) = (new_y, new_dy);
    }
}

pub fn my_newton<F>(f_df: F, init_theta: f64, performance: &mut Performance) -> Option<(f64, f64)>
where
    F: FnMut(f64) -> (f64, f64),
{
    let root = find_root(
        f_df,
        init_theta,
        -THETA_LIMIT,
        THETA_LIMIT,
        THETA_TOL,
        20,
        performance,
    ); // i mean its usually like 3 iters but idk 
    root
}
