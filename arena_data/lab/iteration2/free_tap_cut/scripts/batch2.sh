#!/bin/bash
# repeat check with the rng fix, dp share of a solve, more seeds, chained runs and the covered regime
S="C:/Users/Kenivia/AppData/Local/Temp/claude/c--Users-Kenivia-Documents-VSC-Honing-Forecast/edbf30a9-819d-40c3-be14-f2dcfd60bc48/scratchpad"
DB="$S/out/ft.sqlite"
cd "$S" && python patch_dp.py "$S/ft"
sed -i 's/^mod special;/pub mod special;/' ft/crates/core/src/core/mean/mod.rs
python - "$S/ft" <<'EOF'
import sys
root = sys.argv[1]
p = root + '/crates/core/src/optimizer/neighbour.rs'
t = open(p, encoding='utf-8', newline='').read()
nl = '\r\n' if '\r\n' in t else '\n'
a = 'let reached = 1.0 - self.state_bundle.special_probs()[0];'
assert a in t
t = t.replace(a, 'let reached = 1.0 - self.state_bundle.first_after_merge();')
open(p, 'w', encoding='utf-8', newline='').write(t)
p = root + '/crates/core/src/core/mean/special.rs'
t = open(p, encoding='utf-8', newline='').read()
a = '    /// The free tap outcomes evaluated, with their weights, by `Tuning::special_fold`'
if '\r\n' in t:
    a = a
assert a in t
t = t.replace(a, '''    /// what production stored for no free taps at all
    pub fn first_after_merge(&self) -> f64 {
        let mut probs = self.special_probs().clone();
        if self.tuning.special_fold == 0 {
            eliminate_tail(0..probs.len(), &mut probs, self.tuning.special_tol);
        }
        probs[0]
    }

''' + a)
open(p, 'w', encoding='utf-8', newline='').write(t)
EOF
cd "$S/ft" && cargo build --release -p hf-arena 2>&1 | tail -2
A=target/release/hf-arena.exe
$A --db "$DB" snap ft_b
D=target/arena/bin/ft_b.exe
$D --db "$DB" --set free_taps run ft_b --mode cold --seeds 3 --jobs 16 2>&1 | tail -1
echo REPEAT_DONE
for s in free_taps real_test speed_test; do
  $D --set $s bench --solve 0 --jobs 1 2>&1 | grep '^DP' > "$S/out/dp_$s.tsv"
done
echo DP_DONE
python "$S/high_set.py" "$S/out" "$S/ft/arena_data/sets/ft_high.txt"
V="ft_b ft_b:special_tol=1e-5,special_fold=2 ft_b:special_tol=1e-4,special_fold=2 ft_b:special_tol=3e-4,special_fold=2"
$D --db "$DB" --set ft_high run $V --mode cold --seeds 5 --jobs 16 2>&1 | tail -1
$D --db "$DB" --set quick_dev run $V --mode cold --seeds 3 --first-seed 2 --jobs 16 2>&1 | tail -1
$D --db "$DB" --set real_income_test run $V --mode chained --seeds 3 --jobs 16 2>&1 | tail -1
$D --db "$DB" --set speed_test run $V --mode cold --seeds 3 --jobs 16 2>&1 | tail -1
echo done
