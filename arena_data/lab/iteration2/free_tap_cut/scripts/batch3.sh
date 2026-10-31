#!/bin/bash
# the cut scaled by the mass outside the most likely outcome, on full solves
S="C:/Users/Kenivia/AppData/Local/Temp/claude/c--Users-Kenivia-Documents-VSC-Honing-Forecast/edbf30a9-819d-40c3-be14-f2dcfd60bc48/scratchpad"
DB="$S/out/ft.sqlite"
python - "$S/ft" <<'EOF'
import sys
p = sys.argv[1] + '/crates/core/src/core/mean/special.rs'
t = open(p, encoding='utf-8', newline='').read()
a = 'let mut probs: Vec<f64> = self.special_probs().iter().map(|p| p.max(0.0)).collect();'
assert a in t
t = t.replace('let tol = self.tuning.special_tol;\n', 'let mut tol = self.tuning.special_tol;\n').replace('let tol = self.tuning.special_tol;\r\n', 'let mut tol = self.tuning.special_tol;\r\n')
b = 'if probs.is_empty() {'
assert t.count(b) == 1
t = t.replace(b, '''if self.tuning.special_fold == 4 {
            // the mass that is not the most likely outcome is what the cost is made of
            tol = (tol * (1.0 - probs.iter().fold(0.0f64, |a, &b| a.max(b)))).max(1e-11);
        }
        ''' + b)
open(p, 'w', encoding='utf-8', newline='').write(t)
EOF
cd "$S/ft" && cargo build --release -p hf-arena 2>&1 | tail -2
A=target/release/hf-arena.exe
$A --db "$DB" snap ft_c
D=target/arena/bin/ft_c.exe
V="ft_c ft_c:special_tol=1e-4,special_fold=4 ft_c:special_tol=3e-4,special_fold=4"
$D --db "$DB" --set ft_high run $V --mode cold --seeds 5 --jobs 16 2>&1 | tail -1
$D --db "$DB" --set quick_dev run $V --mode cold --seeds 3 --first-seed 2 --jobs 16 2>&1 | tail -1
$D --db "$DB" --set speed_test run $V --mode cold --seeds 3 --jobs 16 2>&1 | tail -1
$D --db "$DB" --set real_test run $V --mode cold --seeds 5 --jobs 16 2>&1 | tail -1
$D --db "$DB" --set real_income_test run ft_c:special_tol=1e-4,special_fold=4 ft_c:special_tol=3e-4,special_fold=4 --mode chained --seeds 3 --jobs 16 2>&1 | tail -1
echo done
