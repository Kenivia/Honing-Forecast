# Scratch build cut_x: the production cut with the old reach of order moves (1e-7), so only the reach differs from cut_a.
$s = 'arena_data/lab/iteration2/free_tap_cut/scratch.sqlite'
target/release/hf-arena.exe --db $s snap cut_x | Out-Null
foreach ($f in 'chain_0017_t4_n14', 'single_3300_t4_n23') {
    target\arena\bin\cut_x.exe --db $s --set quick_dev run cut_x --mode cold --seeds 10 --jobs 16 --filter $f | Select-Object -Last 1 | Out-Null
}
target\arena\bin\cut_x.exe --db $s --set quick_dev run cut_x:algo=lns,lab_effort=1 --mode cold --seeds 10 --jobs 16 --filter single_1033_serca_n21 | Select-Object -Last 1 | Out-Null
