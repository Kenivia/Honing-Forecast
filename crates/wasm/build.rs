//! Works out the advanced honing distributions of an upgrade not yet started, every strategy
//! of it, so that the browser does not have to. Read by `add_fresh_adv` in `src/lib.rs`.

use hf_core::advanced_honing::compute::compute_adv_dist_wrapper;
use hf_core::advanced_honing::utils::{AdvConfig, AdvDistTriplet};

fn main() {
    // the game gives every upgrade double balls now; one without is worked out in the browser
    let table: Vec<(AdvConfig, AdvDistTriplet)> = [false, true]
        .into_iter()
        .flat_map(|is_30_40| AdvConfig::new(0, 0, false, false, true, is_30_40).strategies())
        .map(|config| (config, compute_adv_dist_wrapper(&config)))
        .collect();
    let path = std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).join("adv_fresh.bin");
    std::fs::write(path, postcard::to_allocvec(&table).unwrap()).unwrap();
}
