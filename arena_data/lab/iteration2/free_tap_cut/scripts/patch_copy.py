# experiment patch for the copy only
import re, sys
root = sys.argv[1]
def sub(path, pairs):
    p = f'{root}/{path}'
    t = open(p, encoding='utf-8', newline='').read()
    crlf = '\r\n' in t
    if crlf:
        t = t.replace('\r\n', '\n')
    for a, b in pairs:
        assert a in t, (path, a)
        t = t.replace(a, b)
    if crlf:
        t = t.replace('\n', '\r\n')
    open(p, 'w', encoding='utf-8', newline='').write(t)

sub('crates/core/src/tuning.rs', [
("""    pub max_coarse_work: usize,

    // optimizer
""", """    pub max_coarse_work: usize,
    /// free tap outcomes under this chance are not evaluated
    pub special_tol: f64,
    /// what becomes of their chance: 0 lost, 1 renormalised, 2 folded onto the nearest outcome
    /// kept, 3 folded keeping the mean count
    pub special_fold: usize,

    // optimizer
"""),
("""            max_coarse_work: 20_000_000,

""", """            max_coarse_work: 20_000_000,
            special_tol: 1e-7,
            special_fold: 0,

"""),
("""            coarse_cells: 64.0,
            ..Tuning::default()
""", """            coarse_cells: 64.0,
            special_tol: 1e-11,
            special_fold: 2,
            ..Tuning::default()
"""),
])

sub('crates/core/src/core/mean/special.rs', [
("""        if !preserve_tail {
            eliminate_tail(0..actual_out.len(), &mut actual_out);
            eliminate_tail(actual_out.len()..0, &mut actual_out);
        }

""", ""),
("""    pub fn special_probs(&self) -> &Vec<f64> {
        // we don't automatically call compute_special here cos that'd need &mut self
        &self.special_cache[&self.prep_output.special_budget][&self.special_state]
    }
""", """    pub fn special_probs(&self) -> &Vec<f64> {
        // we don't automatically call compute_special here cos that'd need &mut self
        &self.special_cache[&self.prep_output.special_budget][&self.special_state]
    }

    /// The free tap outcomes evaluated, with their weights, by `Tuning::special_fold`
    pub fn skip_weights(&self) -> Vec<(usize, f64)> {
        let tol = self.tuning.special_tol;
        let mut probs: Vec<f64> = self.special_probs().iter().map(|p| p.max(0.0)).collect();
        if probs.is_empty() {
            return Vec::new();
        }
        if self.tuning.special_fold == 0 {
            eliminate_tail(0..probs.len(), &mut probs, tol);
            return probs
                .iter()
                .copied()
                .enumerate()
                .filter(|&(_, p)| p >= tol)
                .collect();
        }
        let low = probs.iter().position(|&p| p >= tol).unwrap();
        let high = probs.iter().rposition(|&p| p >= tol).unwrap();
        let mut out: Vec<(usize, f64)> = (low..=high).map(|k| (k, probs[k])).collect();
        let last = out.len() - 1;
        let below: f64 = probs[..low].iter().sum();
        let above: f64 = probs[high + 1..].iter().sum();
        match self.tuning.special_fold {
            1 => {
                let kept: f64 = out.iter().map(|x| x.1).sum();
                out.iter_mut().for_each(|x| x.1 /= kept);
            }
            _ => {
                out[0].1 += below;
                out[last].1 += above;
                if self.tuning.special_fold == 3 && last > 0 {
                    let low_moved: f64 = (0..low).map(|k| probs[k] * (low - k) as f64).sum();
                    let high_moved: f64 = (high + 1..probs.len())
                        .map(|k| probs[k] * (k - high) as f64)
                        .sum();
                    out[0].1 += low_moved;
                    out[1].1 -= low_moved;
                    out[last].1 += high_moved;
                    out[last - 1].1 -= high_moved;
                }
            }
        }
        out
    }
"""),
("use crate::constants::{IGNORE_PROB_TOL, SPECIAL_TOL};\n", "use crate::constants::IGNORE_PROB_TOL;\n"),
("fn eliminate_tail(range: Range<usize>, actual_out: &mut Vec<f64>) {", "fn eliminate_tail(range: Range<usize>, actual_out: &mut Vec<f64>, tol: f64) {"),
("        if this > SPECIAL_TOL {\n", "        if this > tol {\n"),
("        if cur_sum > SPECIAL_TOL {\n", "        if cur_sum > tol {\n"),
])

sub('crates/core/src/core/mean/average.rs', [
("""    fn live_skips(&self) -> Vec<(usize, f64)> {
        self.special_probs()
            .iter()
            .copied()
            .enumerate()
            .filter(|&(_, special_prob)| special_prob >= SPECIAL_TOL)
            .collect()
    }
""", """    fn live_skips(&self) -> Vec<(usize, f64)> {
        self.skip_weights()
    }

    /// The average gold given exactly these free tap outcomes, one each
    pub fn outcome_golds(&mut self, outcomes: &[usize], probe: &mut Probe) -> Vec<f64> {
        self.update_prob_dist();
        self.update_cost_dist();
        self.compute_special_probs(false);
        let mut memo = std::mem::take(&mut self.gold_memo);
        let tuning_key = self.tuning.evaluator_key();
        let plan_bands = &self.prep_output.bands[self.prep_output.optimizer_plan];
        let out = outcomes
            .iter()
            .map(|&k| {
                let skips = [(k, 1.0)];
                let sums = if self.tuning.max_memo_size > 0 {
                    self.memo_sums(&skips)
                } else {
                    Vec::new()
                };
                plan_bands
                    .iter()
                    .enumerate()
                    .map(|(row, pairs)| {
                        self.material_average_gold(
                            &mut memo,
                            row as i64,
                            &skips,
                            pairs,
                            sums.get(row).map(|sums| &sums[..]),
                            tuning_key,
                            probe,
                        )
                    })
                    .sum()
            })
            .collect();
        self.gold_memo = memo;
        out
    }
"""),
("use crate::constants::{FLOAT_TOL, SPECIAL_TOL, float_tol};", "use crate::constants::{FLOAT_TOL, float_tol};"),
])

sub('crates/core/src/optimizer/shadow.rs', [
("""        let skips: Vec<(usize, f64)> = self
            .special_probs()
            .iter()
            .copied()
            .enumerate()
            .filter(|&(_, weight)| weight >= SPECIAL_TOL)
            .collect();
""", """        let skips: Vec<(usize, f64)> = self.skip_weights();
"""),
("use crate::constants::{FLOAT_TOL, SPECIAL_TOL};", "use crate::constants::FLOAT_TOL;"),
])

sub('crates/core/src/optimizer/response.rs', [
("""        let skips: Vec<(usize, f64)> = state_bundle
            .special_probs()
            .iter()
            .copied()
            .enumerate()
            .filter(|&(skip, weight)| weight >= SPECIAL_TOL && skip <= place)
            .collect();
""", """        let skips: Vec<(usize, f64)> = state_bundle
            .skip_weights()
            .into_iter()
            .filter(|&(skip, _)| skip <= place)
            .collect();
"""),
("use crate::constants::{ARTISAN_MULTIPLIER, FLOAT_TOL, SPECIAL_TOL};", "use crate::constants::{ARTISAN_MULTIPLIER, FLOAT_TOL};"),
])

sub('crates/core/src/optimizer/neighbour.rs', [
("""            let reach = self
                .state_bundle
                .special_probs()
                .iter()
                .rposition(|&chance| chance >= SPECIAL_TOL)
                .unwrap_or(0);
""", """            let reach = self.state_bundle.skip_weights().last().map_or(0, |x| x.0);
"""),
("use crate::constants::SPECIAL_TOL;\n", ""),
])

sub('crates/core/src/optimizer/simulated_annealing.rs', [
("""            let reach = state_bundle
                .special_probs()
                .iter()
                .rposition(|&chance| chance >= SPECIAL_TOL)
                .unwrap_or(0);
""", """            let reach = state_bundle.skip_weights().last().map_or(0, |x| x.0);
"""),
("use crate::constants::SPECIAL_TOL;\n", ""),
])

sub('crates/arena/src/main.rs', [
("mod space;\n", "mod space;\nmod special_dump;\n"),
("""    /// time the evaluator alone
    Bench {
""", """    /// free tap chances and the gold of every outcome, per state
    SpecialDump {
        #[command(flatten)]
        args: special_dump::Args,
    },
    /// time the evaluator alone
    Bench {
"""),
("""        Cmd::Bench { args } => {
""", """        Cmd::SpecialDump { args } => {
            let scenarios = scenarios(&cli, &args.filter);
            special_dump::run(&scenarios, args);
        }
        Cmd::Bench { args } => {
"""),
])
