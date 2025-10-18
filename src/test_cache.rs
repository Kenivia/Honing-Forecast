#[cfg(test)]
use serde_json;
#[cfg(test)]
use std::fs;
#[cfg(test)]
use std::path::PathBuf;

/// Calculate a hash from multiple hashable values
#[macro_export]
#[cfg(test)]
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
