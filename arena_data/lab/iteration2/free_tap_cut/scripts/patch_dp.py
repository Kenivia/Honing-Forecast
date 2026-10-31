# second experiment patch: how long the free tap dp takes inside a solve
import sys
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

sub('crates/core/src/core/mean/special.rs', [
("""        let prep_output = &self.prep_output;

        let m = self
""", """        let _timed = DpTimer(std::time::Instant::now());
        let prep_output = &self.prep_output;

        let m = self
"""),
("""impl StateBundle {
    pub fn special_probs(&self) -> &Vec<f64> {""", """pub static DP_NS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
pub static DP_CALLS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
struct DpTimer(std::time::Instant);
impl Drop for DpTimer {
    fn drop(&mut self) {
        let spent = self.0.elapsed().as_nanos() as u64;
        DP_NS.fetch_add(spent, std::sync::atomic::Ordering::Relaxed);
        DP_CALLS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
}

impl StateBundle {
    pub fn special_probs(&self) -> &Vec<f64> {"""),
])

sub('crates/arena/src/bench.rs', [
("""                    let start = Instant::now();
                    let solved = hf_core::optimizer::lab::solve(&mut rng, base, None, &mut probe);
                    return vec![(
                        name.clone(),
                        solved.best.metric,
                        start.elapsed().as_secs_f64(),
                        probe.data.counts.to_vec(),
                        f64::NAN,
                    )];
""", """                    use hf_core::core::mean::special::{DP_CALLS, DP_NS};
                    use std::sync::atomic::Ordering::Relaxed;
                    let (ns, calls) = (DP_NS.load(Relaxed), DP_CALLS.load(Relaxed));
                    let start = Instant::now();
                    let solved = hf_core::optimizer::lab::solve(&mut rng, base, None, &mut probe);
                    let wall = start.elapsed().as_secs_f64();
                    println!(
                        "DP\\t{name}\\t{wall:.4}\\t{:.4}\\t{}\\t{}",
                        (DP_NS.load(Relaxed) - ns) as f64 / 1e9,
                        DP_CALLS.load(Relaxed) - calls,
                        solved.best.prep_output.special_budget,
                    );
                    return vec![(
                        name.clone(),
                        solved.best.metric,
                        wall,
                        probe.data.counts.to_vec(),
                        f64::NAN,
                    )];
"""),
])
