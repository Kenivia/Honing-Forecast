// histogram utilities for cost distributions

/// Compute fixed-width histogram for a single cost index over Monte Carlo samples.
/// Returns (counts[num_bins], min_value, max_value).
pub fn histogram_for_cost_index(
    cost_data: &Vec<Vec<i64>>,
    cost_index: usize,
    num_bins: usize,
) -> (Vec<i64>, i64, i64) {
    let n: usize = cost_data.len();
    let mut values: Vec<i64> = Vec::with_capacity(n);
    for row in cost_data.iter() {
        // cost_data rows are at least 7 long in this codebase
        values.push(row[cost_index]);
    }
    if values.is_empty() {
        return (vec![0i64; num_bins], 0, 0);
    }
    values.sort_unstable();
    let min_val: i64 = values[0];
    let max_val: i64 = *values.last().unwrap();
    let mut counts: Vec<i64> = vec![0i64; num_bins];

    if min_val == max_val {
        // All samples identical; put them in the last bucket to avoid div-by-zero
        counts[num_bins - 1] = n as i64;
        return (counts, min_val, max_val);
    }

    let range_f64: f64 = (max_val - min_val) as f64;
    for v in values.iter() {
        let pos: f64 = (*v - min_val) as f64 / range_f64;
        let mut bin: usize = (pos * num_bins as f64).floor() as usize;
        if bin >= num_bins {
            bin = num_bins - 1;
        }
        counts[bin] += 1;
    }
    (counts, min_val, max_val)
}

/// Compute histograms for the 7 cost types (indices 0..6).
/// Returns (counts_7xB, mins_7, maxs_7)
pub fn histograms_for_all_costs(
    cost_data: &Vec<Vec<i64>>,
    num_bins: usize,
) -> (Vec<Vec<i64>>, Vec<i64>, Vec<i64>) {
    let mut all_counts: Vec<Vec<i64>> = Vec::with_capacity(7);
    let mut mins: Vec<i64> = Vec::with_capacity(7);
    let mut maxs: Vec<i64> = Vec::with_capacity(7);
    for idx in 0..7 {
        let (counts, min_v, max_v) = histogram_for_cost_index(cost_data, idx, num_bins);
        all_counts.push(counts);
        mins.push(min_v);
        maxs.push(max_v);
    }
    (all_counts, mins, maxs)
}
