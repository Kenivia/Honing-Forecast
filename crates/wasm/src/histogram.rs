// use hf_core::constants::BUCKET_COUNT;
// // use hf_core::helpers::transpose_vec_of_vecs;

// use hf_core::performance::Performance;
// use hf_core::state_bundle::StateBundle;

// pub fn histogram_for_cost_index(
//     cost_data_row: &mut Vec<i64>,
//     num_bins: usize,
//     highest: i64,
// ) -> Vec<i64> {
//     if cost_data_row.is_empty() {
//         return vec![0i64; num_bins];
//     }
//     cost_data_row.sort_unstable();
//     let min_val: i64 = 0;
//     let max_val: i64 = highest;
//     let mut counts: Vec<i64> = vec![0i64; num_bins];

//     if min_val == max_val {
//         // All samples identical; put them in the last bucket to avoid div-by-zero
//         counts[num_bins - 1] = cost_data_row.len() as i64;
//         return counts;
//     }

//     let range_f64: f64 = (max_val - min_val) as f64;
//     for v in cost_data_row.iter() {
//         let pos: f64 = (*v - min_val) as f64 / range_f64;
//         let mut bin: usize = (pos * num_bins as f64).floor() as usize;
//         if bin >= num_bins {
//             bin = num_bins - 1;
//         }
//         counts[bin] += 1;
//     }
//     counts
// }

// Compute histograms for the 7 cost types (indices 0..6).
// Returns (`counts_7xB`, `mins_7`, `maxs_7`)
// pub fn histograms_for_all_costs(
//     state_bundle: &StateBundle,
//     num_bins: usize,
//     highest: &[i64],
//     data_size: usize,
// ) -> Vec<Vec<i64>> {
//     let mut all_counts: Vec<Vec<i64>> = Vec::with_capacity(7);
//     let mut dummy_performance = Performance::new();
//     for support_index in 0..7_i64 {
//         let mut prev_prob = 0.0;
//         let mut counts: Vec<i64> = Vec::with_capacity(num_bins);
//         if highest[support_index as usize] == 0 {
//             let mut out = vec![0; num_bins - 1];
//             out.push(data_size as i64);
//             all_counts.push(out);
//             continue;
//         }
//         for i in 0..num_bins {
//             let effective_budget =
//                 (i * highest[support_index as usize] as usize) as f64 / (num_bins - 1) as f64;
//             let this_prob = state_bundle.honing_sa_wrapper(
//                 support_index,
//                 effective_budget,
//                 &mut dummy_performance,
//             );
//             counts.push(((this_prob - prev_prob) * data_size as f64).round() as i64);
//             prev_prob = this_prob;
//         }
//         all_counts.push(counts);
//     }
//     all_counts
// }
// #[derive(Debug)]
// pub struct HistogramOutputs {
//     pub hist_counts: Vec<Vec<i64>>,
//     pub hist_mins: Vec<i64>,
//     pub hist_maxs: Vec<i64>,
// }
// pub fn prep_histogram(
//     state_bundle: &StateBundle,
//     hist_bins: usize,
//     data_size: usize,
// ) -> HistogramOutputs {
//     let bins: usize = hist_bins.min(BUCKET_COUNT).max(1);
//     let hist_maxs: Vec<i64> = state_bundle.pity();
//     let hist_counts: Vec<Vec<i64>> =
//         histograms_for_all_costs(state_bundle, bins, &hist_maxs, data_size);

//     HistogramOutputs {
//         hist_counts,
//         hist_mins: vec![0_i64; 7],
//         hist_maxs,
//     }
// }
