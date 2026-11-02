# Same build, 10 seeds: the production cut against one 1000 times finer, on the payloads that moved in batch a.
$s = 'arena_data/lab/iteration2/free_tap_cut/scratch.sqlite'
target/release/hf-arena.exe --db $s snap cut_a | Out-Null
function pair($set, $filter, $base) {
    $tail = if ($base) { ",$base" } else { '' }
    $fine = if ($base) { "cut_a:$base,special_tol=1e-7" } else { 'cut_a:special_tol=1e-7' }
    $plain = if ($base) { "cut_a:$base" } else { 'cut_a' }
    target\arena\bin\cut_a.exe --db $s --set $set run $plain $fine --mode cold --seeds 10 --jobs 16 --filter $filter | Select-Object -Last 1 | Out-Null
    "done $set $filter $base"
}
pair free_taps single_0071_serca_n19 'algo=price,lab_effort=1'
pair speed_test single_1208_t4_n14 'algo=price,lab_effort=1'
pair quick_dev single_1033_serca_n21 'algo=lns,lab_effort=1'
pair quick_dev single_0161_t4_n39 'algo=lns,lab_effort=1'
pair quick_dev single_3300_t4_n23 ''
pair quick_dev single_3300_t4_n23 'algo=surrogate,lab_effort=1'
pair quick_dev chain_0017_t4_n14 ''
