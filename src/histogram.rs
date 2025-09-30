pub fn transpose_vec_of_vecs(matrix: &[Vec<i64>]) -> Vec<Vec<i64>> {
    if matrix.is_empty() || matrix[0].is_empty() {
        return Vec::new();
    }

    let rows: usize = matrix.len();
    let cols: usize = matrix[0].len();

    let mut transposed: Vec<Vec<i64>> = vec![vec![0_i64; rows]; cols]; // Initialize with default values

    for r in 0..rows {
        for c in 0..cols {
            transposed[c][r] = matrix[r][c];
        }
    }
    transposed
}

pub fn histogram_for_cost_index(
    cost_data_row: &mut Vec<i64>,
    num_bins: usize,
    highest: i64,
) -> Vec<i64> {
    if cost_data_row.is_empty() {
        return vec![0i64; num_bins];
    }
    cost_data_row.sort_unstable();
    let min_val: i64 = 0;
    let max_val: i64 = highest;
    let mut counts: Vec<i64> = vec![0i64; num_bins];

    if min_val == max_val {
        // All samples identical; put them in the last bucket to avoid div-by-zero
        counts[num_bins - 1] = cost_data_row.len() as i64;
        return counts;
    }

    let range_f64: f64 = (max_val - min_val) as f64;
    for v in cost_data_row.iter() {
        let pos: f64 = (*v - min_val) as f64 / range_f64;
        let mut bin: usize = (pos * num_bins as f64).floor() as usize;
        if bin >= num_bins {
            bin = num_bins - 1;
        }
        counts[bin] += 1;
    }
    counts
}

/// Compute histograms for the 7 cost types (indices 0..6).
/// Returns (counts_7xB, mins_7, maxs_7)
pub fn histograms_for_all_costs(
    cost_data: &[Vec<i64>],
    num_bins: usize,
    highest: &[i64],
) -> Vec<Vec<i64>> {
    let mut all_counts: Vec<Vec<i64>> = Vec::with_capacity(7);
    let mut transposed: Vec<Vec<i64>> = transpose_vec_of_vecs(cost_data);
    for idx in 0..7 {
        let counts: Vec<i64> =
            histogram_for_cost_index(&mut transposed[idx], num_bins, highest[idx]);
        all_counts.push(counts);
    }
    all_counts
}
