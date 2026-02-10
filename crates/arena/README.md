# Arena

This is where the optimizer engine is tested. You can add another version in the engine folder in core, add it in `mod.rs` and `Cargo.toml` (alongside all the others), then test it via `cargo run --package hf-arena --bin hf-arena --no-default-features --features YOUR_VERSION  --profile release -- ./crates/arena/test_payloads` (This should be ran from the project root).

I don't think too many assumptions are made about the behaviour of the neighbour function, as long as it outputs valid states it should be okay. Specifically:

- special state should be a permutation of the indices of upgrade_arr (btw upgrade_arr should never be re-ordered)
- you should not change the length of state, it should always have the same length as clean_prob_dist_len
- don't put invalid books in states ig

I called it arena because I was going to make a more adversaial like elo system to see which engine performs best (like stockfish), but realized that looking at graphs are just far more informative and useful. I can't be bothered to change the name so here we are.
