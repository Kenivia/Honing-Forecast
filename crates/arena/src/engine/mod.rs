// TODO WRITE A SCRIPT TO ADD A LINE TO THIS
// =========== FEATURES START ============
mod v1;
mod v2;

cfg_if::cfg_if! {
    if #[cfg(feature = "v1")] {
        pub use v1::*;
        pub static ACTIVE_FEATURE :&str= "v1";
    } else if #[cfg(feature = "v2")] {
        pub use v2::*;
        pub static ACTIVE_FEATURE :&str= "v2";
    }
}

// pub use impl_::*;
