// Put this in your crate (e.g. near the bottom of src/lib.rs) so it's only compiled for tests.

use crate::parser::Upgrade;

use serde_json;
use std::fmt::Debug;
use std::fs;
use std::path::PathBuf;
/// Tolerance for float comparisons
pub const EPSILON: f64 = 1e-7;

/// Trait that enables approximate / deep equality assertions in tests
pub trait AssertApproxEq {
    /// ctx is used to include context in panic messages (like file:line)
    fn assert_approx_eq(&self, other: &Self, ctx: &str);
}

// Helper for f64 comparisons using EPSILON
fn my_float_eq(a: f64, b: f64) -> bool {
    if a == b {
        // catches infinities and exact equality
        true
    } else {
        (a - b).abs() <= EPSILON
    }
}

// Impl for f64
impl AssertApproxEq for f64 {
    fn assert_approx_eq(&self, other: &Self, ctx: &str) {
        if !my_float_eq(*self, *other) {
            panic!(
                "Assertion failed (f64 approx): {}\n  left = {:e}\n right = {:e}\n epsilon = {}",
                ctx, self, other, EPSILON
            );
        } else {
            ()
        }
    }
}

// Integers and usize: exact equality
macro_rules! impl_int {
        ($($t:ty),*) => {
            $(
                impl AssertApproxEq for $t {
                    fn assert_approx_eq(&self, other: &Self, ctx: &str) {
                        if self != other {
                            panic!(
                                "Assertion failed (int eq): {}\n  left = {:?}\n right = {:?}",
                                ctx, self, other
                            );
                        }
                    }
                }
            )*
        };
    }
impl_int!(
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize
);

impl AssertApproxEq for bool {
    fn assert_approx_eq(&self, other: &Self, ctx: &str) {
        if self != other {
            panic!(
                "Assertion failed (bool eq): {}\n  left = {:?}\n right = {:?}",
                ctx, self, other
            );
        }
    }
}

// Arrays with fixed length for your costs [i64; 7]
impl AssertApproxEq for [i64; 7] {
    fn assert_approx_eq(&self, other: &Self, ctx: &str) {
        for (i, (a, b)) in self.iter().zip(other.iter()).enumerate() {
            if a != b {
                panic!(
                    "Assertion failed (array [i64;7] at index {}): {}\n  left = {:?}\n right = {:?}",
                    i, ctx, self, other
                );
            }
        }
    }
}
impl AssertApproxEq for [f64] {
    fn assert_approx_eq(&self, other: &Self, ctx: &str) {
        for (i, (a, b)) in self.iter().zip(other.iter()).enumerate() {
            if !my_float_eq(*a, *b) {
                panic!(
                    "Assertion failed (array [f64] at index {}): {}\n  left = {:?}\n right = {:?}",
                    i, ctx, self, other
                );
            }
        }
    }
}

// Vec<T> where T: AssertApproxEq
impl<T> AssertApproxEq for Vec<T>
where
    T: AssertApproxEq + Debug,
{
    fn assert_approx_eq(&self, other: &Self, ctx: &str) {
        if self.len() != other.len() {
            panic!(
                "Assertion failed (vec length): {}\n  left.len() = {}\n right.len() = {}",
                ctx,
                self.len(),
                other.len()
            );
        }
        for (i, (a, b)) in self.iter().zip(other.iter()).enumerate() {
            let subctx = format!("{} (vec index {})", ctx, i);
            a.assert_approx_eq(b, &subctx);
        }
    }
}

// Option<T>
impl<T> AssertApproxEq for Option<T>
where
    T: AssertApproxEq + Debug,
{
    fn assert_approx_eq(&self, other: &Self, ctx: &str) {
        match (self, other) {
            (Some(a), Some(b)) => a.assert_approx_eq(b, ctx),
            (None, None) => {}
            _ => panic!(
                "Assertion failed (option mismatch): {}\n left = {:?}\nright = {:?}",
                ctx, self, other
            ),
        }
    }
}

// If you want arrays of floats or other lengths, add impls similarly.
// ----
// Now implement AssertApproxEq for your Upgrade struct.
// Update the path `crate::Upgrade` if Upgrade lives in a submodule.

impl AssertApproxEq for Upgrade {
    fn assert_approx_eq(&self, other: &Self, ctx: &str) {
        // booleans / integers / usize by exact equality
        if self.is_normal_honing != other.is_normal_honing {
            panic!(
                "{}: is_normal_honing mismatch: {:?} != {:?}",
                ctx, self.is_normal_honing, other.is_normal_honing
            );
        }

        // prob_dist: Vec<f64>
        self.prob_dist
            .assert_approx_eq(&other.prob_dist, &format!("{} -> prob_dist", ctx));

        // original_prob_dist: Vec<f64>
        self.original_prob_dist.assert_approx_eq(
            &other.original_prob_dist,
            &format!("{} -> original_prob_dist", ctx),
        );

        // base_chance: f64
        self.base_chance
            .assert_approx_eq(&other.base_chance, &format!("{} -> base_chance", ctx));

        // costs: [i64;7]
        self.costs
            .assert_approx_eq(&other.costs, &format!("{} -> costs", ctx));

        // one_juice_cost
        self.one_juice_cost
            .assert_approx_eq(&other.one_juice_cost, &format!("{} -> one_juice_cost", ctx));

        // adv_juice_cost: Vec<f64>
        self.adv_juice_cost
            .assert_approx_eq(&other.adv_juice_cost, &format!("{} -> adv_juice_cost", ctx));

        // special_cost
        self.special_cost
            .assert_approx_eq(&other.special_cost, &format!("{} -> special_cost", ctx));

        // juice_values: Vec<f64>
        self.juice_values
            .assert_approx_eq(&other.juice_values, &format!("{} -> juice_values", ctx));

        // prob_dist_len: usize
        self.prob_dist_len
            .assert_approx_eq(&other.prob_dist_len, &format!("{} -> prob_dist_len", ctx));

        // is_weapon
        self.is_weapon
            .assert_approx_eq(&other.is_weapon, &format!("{} -> is_weapon", ctx));

        // artisan_rate: f64
        self.artisan_rate
            .assert_approx_eq(&other.artisan_rate, &format!("{} -> artisan_rate", ctx));

        // tap_offset: i64
        self.tap_offset
            .assert_approx_eq(&other.tap_offset, &format!("{} -> tap_offset", ctx));

        // upgrade_plus_num: usize
        self.upgrade_plus_num.assert_approx_eq(
            &other.upgrade_plus_num,
            &format!("{} -> upgrade_plus_num", ctx),
        );

        // special_value: f64
        self.special_value
            .assert_approx_eq(&other.special_value, &format!("{} -> special_value", ctx));
    }
}
// --- impl for ChanceToCostOut ---
use crate::chance_to_cost::ChanceToCostOut;

impl AssertApproxEq for ChanceToCostOut {
    fn assert_approx_eq(&self, other: &Self, ctx: &str) {
        // hist_counts: Vec<Vec<i64>>
        self.hist_counts
            .assert_approx_eq(&other.hist_counts, &format!("{} -> hist_counts", ctx));

        // hist_mins: Vec<i64>
        self.hist_mins
            .assert_approx_eq(&other.hist_mins, &format!("{} -> hist_mins", ctx));

        // hist_maxs: Vec<i64>
        self.hist_maxs
            .assert_approx_eq(&other.hist_maxs, &format!("{} -> hist_maxs", ctx));

        // hundred_budgets: Vec<Vec<i64>>
        self.hundred_budgets.assert_approx_eq(
            &other.hundred_budgets,
            &format!("{} -> hundred_budgets", ctx),
        );

        // hundred_chances: Vec<f64>
        self.hundred_chances.assert_approx_eq(
            &other.hundred_chances,
            &format!("{} -> hundred_chances", ctx),
        );
    }
}

// --- impl for CostToChanceOut ---
use crate::cost_to_chance::CostToChanceOut;

impl AssertApproxEq for CostToChanceOut {
    fn assert_approx_eq(&self, other: &Self, ctx: &str) {
        // chance: f64
        self.chance
            .assert_approx_eq(&other.chance, &format!("{} -> chance", ctx));

        // reasons: Vec<f64>
        self.reasons
            .assert_approx_eq(&other.reasons, &format!("{} -> reasons", ctx));

        // hist_counts: Vec<Vec<i64>>
        self.hist_counts
            .assert_approx_eq(&other.hist_counts, &format!("{} -> hist_counts", ctx));

        // hist_mins: Vec<i64>
        self.hist_mins
            .assert_approx_eq(&other.hist_mins, &format!("{} -> hist_mins", ctx));

        // hist_maxs: Vec<i64>
        self.hist_maxs
            .assert_approx_eq(&other.hist_maxs, &format!("{} -> hist_maxs", ctx));

        // upgrade_strings: Vec<String> (exact equality per element)
        if self.upgrade_strings.len() != other.upgrade_strings.len() {
            panic!(
                "{} -> upgrade_strings length mismatch: {} != {}",
                ctx,
                self.upgrade_strings.len(),
                other.upgrade_strings.len()
            );
        }
        for (i, (a, b)) in self
            .upgrade_strings
            .iter()
            .zip(other.upgrade_strings.iter())
            .enumerate()
        {
            if a != b {
                panic!(
                    "{} -> upgrade_strings[{}] mismatch:\n  left = {:?}\n right = {:?}",
                    ctx, i, a, b
                );
            }
        }

        // juice_strings_armor: Vec<String>
        if self.juice_strings_armor.len() != other.juice_strings_armor.len() {
            panic!(
                "{} -> juice_strings_armor length mismatch: {} != {}",
                ctx,
                self.juice_strings_armor.len(),
                other.juice_strings_armor.len()
            );
        }
        for (i, (a, b)) in self
            .juice_strings_armor
            .iter()
            .zip(other.juice_strings_armor.iter())
            .enumerate()
        {
            if a != b {
                panic!(
                    "{} -> juice_strings_armor[{}] mismatch:\n  left = {:?}\n right = {:?}",
                    ctx, i, a, b
                );
            }
        }

        // juice_strings_weapon: Vec<String>
        if self.juice_strings_weapon.len() != other.juice_strings_weapon.len() {
            panic!(
                "{} -> juice_strings_weapon length mismatch: {} != {}",
                ctx,
                self.juice_strings_weapon.len(),
                other.juice_strings_weapon.len()
            );
        }
        for (i, (a, b)) in self
            .juice_strings_weapon
            .iter()
            .zip(other.juice_strings_weapon.iter())
            .enumerate()
        {
            if a != b {
                panic!(
                    "{} -> juice_strings_weapon[{}] mismatch:\n  left = {:?}\n right = {:?}",
                    ctx, i, a, b
                );
            }
        }

        // budgets_red_remaining: i64
        self.budgets_red_remaining.assert_approx_eq(
            &other.budgets_red_remaining,
            &format!("{} -> budgets_red_remaining", ctx),
        );

        // budgets_blue_remaining: i64
        self.budgets_blue_remaining.assert_approx_eq(
            &other.budgets_blue_remaining,
            &format!("{} -> budgets_blue_remaining", ctx),
        );

        // hundred_gold_costs: Vec<i64>
        self.hundred_gold_costs.assert_approx_eq(
            &other.hundred_gold_costs,
            &format!("{} -> hundred_gold_costs", ctx),
        );

        // chance_if_buy: f64
        self.chance_if_buy
            .assert_approx_eq(&other.chance_if_buy, &format!("{} -> chance_if_buy", ctx));
    }
}

// --- impl for CostToChanceArrOut ---
use crate::cost_to_chance::CostToChanceArrOut;

impl AssertApproxEq for CostToChanceArrOut {
    fn assert_approx_eq(&self, other: &Self, ctx: &str) {
        // final_chances: Vec<f64>
        self.final_chances
            .assert_approx_eq(&other.final_chances, &format!("{} -> final_chances", ctx));

        // typed_fail_counters: Vec<Vec<f64>>
        self.typed_fail_counters.assert_approx_eq(
            &other.typed_fail_counters,
            &format!("{} -> typed_fail_counters", ctx),
        );

        // budgets_red_remaining: i64
        self.budgets_red_remaining.assert_approx_eq(
            &other.budgets_red_remaining,
            &format!("{} -> budgets_red_remaining", ctx),
        );

        // budgets_blue_remaining: i64
        self.budgets_blue_remaining.assert_approx_eq(
            &other.budgets_blue_remaining,
            &format!("{} -> budgets_blue_remaining", ctx),
        );

        // buy_chances: Vec<f64>
        self.buy_chances
            .assert_approx_eq(&other.buy_chances, &format!("{} -> buy_chances", ctx));
    }
}

// Finally the macro wrapper; it captures file:line to include context in panic messages
#[cfg(test)]
#[macro_export]
macro_rules! my_assert {
    ($left:expr, $right:expr) => {{
        // We import the trait method
        // left and right are evaluated once each
        let left_val = &$left;
        let right_val = &$right;
        use crate::test_utils::AssertApproxEq;
        // Build a short context (file:line)
        let ctx = format!("{}:{}: my_assert!", file!(), line!());
        // Call the trait method that will panic on mismatch
        AssertApproxEq::assert_approx_eq(left_val, right_val, &ctx);
    }};
}

#[cfg(test)]
#[macro_export]
macro_rules! calculate_hash {
    ($($input:expr),*) => {{
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        $( $input.hash(&mut hasher); )*
        format!("{:x}", hasher.finish())
    }};
}
#[cfg(test)]
/// Get the filename for a cached test case
fn get_cache_filename(test_name: &str, hash: &String) -> String {
    format!("{}_{}.json", test_name, hash)
}
#[cfg(test)]
/// Read cached data from a test case file
pub fn read_cached_data<T>(test_name: &str, hash: &String) -> Option<T>
where
    T: for<'de> serde::Deserialize<'de>,
{
    let cache_dir = PathBuf::from("test_cases");
    let filename = get_cache_filename(test_name, hash);
    let file_path = cache_dir.join(filename);

    if !file_path.exists() {
        return None;
    }

    match fs::read_to_string(&file_path) {
        Ok(content) => match serde_json::from_str::<T>(&content) {
            Ok(data) => Some(data),
            Err(e) => {
                eprintln!(
                    "Failed to parse cached test case {}: {}",
                    file_path.display(),
                    e
                );
                None
            }
        },
        Err(e) => {
            eprintln!(
                "Failed to read cached test case {}: {}",
                file_path.display(),
                e
            );
            None
        }
    }
}
#[cfg(test)]
/// Write cached data to a test case file
pub fn write_cached_data<T>(test_name: &str, hash: &String, data: &T)
where
    T: serde::Serialize,
{
    let cache_dir = PathBuf::from("test_cases");

    // Create the test_cases directory if it doesn't exist
    if !cache_dir.exists() {
        fs::create_dir_all(&cache_dir).unwrap();
    }

    let filename = get_cache_filename(test_name, hash);
    let file_path = cache_dir.join(filename);

    let json_content = serde_json::to_string_pretty(data).unwrap();
    fs::write(&file_path, json_content).unwrap();

    println!("Cached test case written to: {}", file_path.display());
}
