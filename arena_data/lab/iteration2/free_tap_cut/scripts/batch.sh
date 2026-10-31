#!/bin/bash
# pristine HEAD against the patched copy, then solves per threshold and policy; only the scratch db is written
S="C:/Users/Kenivia/AppData/Local/Temp/claude/c--Users-Kenivia-Documents-VSC-Honing-Forecast/edbf30a9-819d-40c3-be14-f2dcfd60bc48/scratchpad"
R="C:/Users/Kenivia/Documents/VSC/Honing-Forecast"
DB="$S/out/ft.sqlite"
mkdir -p "$S/ft0"
cd "$R" && git archive HEAD crates Cargo.toml Cargo.lock .cargo public | tar -x -C "$S/ft0"
cd "$S/ft0" && CARGO_TARGET_DIR="$S/ft/target0" cargo build --release -p hf-arena 2>&1 | tail -1
cd "$S/ft"
A=target/release/hf-arena.exe
$A --db "$DB" snap ft_a
$A --db "$DB" snap head
cp "$S/ft/target0/release/hf-arena.exe" target/arena/bin/head.exe
D=target/arena/bin/ft_a.exe
# the copy with nothing set must repeat HEAD
$D --db "$DB" --set small run head ft_a --mode cold --seeds 3 --jobs 16 2>&1 | tail -3
$D --db "$DB" --set free_taps run head ft_a --mode cold --seeds 3 --jobs 16 2>&1 | tail -3
V="ft_a ft_a:special_tol=1e-6 ft_a:special_tol=1e-5 ft_a:special_fold=2 ft_a:special_tol=1e-5,special_fold=2 ft_a:special_tol=1e-4,special_fold=2 ft_a:special_tol=1e-3,special_fold=2"
$D --db "$DB" --set free_taps run $V --mode cold --seeds 5 --first-seed 3 --jobs 16 2>&1 | tail -2
$D --db "$DB" --set real_test run $V --mode cold --seeds 5 --jobs 16 2>&1 | tail -2
$D --db "$DB" --set quick_dev run $V --mode cold --seeds 2 --jobs 16 2>&1 | tail -2
echo done
