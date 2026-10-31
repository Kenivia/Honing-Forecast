#!/bin/bash
# evaluation time with the memo off, one thread, per threshold and policy
S="C:/Users/Kenivia/AppData/Local/Temp/claude/c--Users-Kenivia-Documents-VSC-Honing-Forecast/edbf30a9-819d-40c3-be14-f2dcfd60bc48/scratchpad"
cd "$S/ft"
A=target/release/hf-arena.exe
for set in free_taps real_test speed_test; do
  for v in "1e-7 0" "1e-6 0" "1e-5 0" "1e-7 2" "1e-5 2" "1e-4 2" "1e-3 2"; do
    set -- $v
    $A --set $set bench --states "$S/out/best.json" --tuning max_memo_size=0,special_tol=$1,special_fold=$2 --out "$S/out/b_${set}_$1_$2.tsv" --jobs 1 --seconds 0.04 | head -1 | sed "s/^/$set $1 $2 /"
  done
done
