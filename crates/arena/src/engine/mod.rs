// TODO WRITE A SCRIPT TO ADD A LINE TO THIS
// =========== FEATURES START ============
mod v1;
mod v2;
mod v3;
mod v4;
mod v5;
mod v6;
cfg_if::cfg_if! {
    if #[cfg(feature = "v1")] {
        pub use v1::*;
        pub static ACTIVE_FEATURE :&str= "v1";
    } else if #[cfg(feature = "v2")] {
        pub use v2::*;
        pub static ACTIVE_FEATURE :&str= "v2";
    }else if #[cfg(feature = "v3")] {
        pub use v3::*;
        pub static ACTIVE_FEATURE :&str= "v3";
    }else if #[cfg(feature = "v4")] {
        pub use v4::*;
        pub static ACTIVE_FEATURE :&str= "v4";
    }else if #[cfg(feature = "v5")] {
        pub use v5::*;
        pub static ACTIVE_FEATURE :&str= "v5";
    }
    else if #[cfg(feature = "v6")] {
        pub use v6::*;
        pub static ACTIVE_FEATURE :&str= "v6";
    }
}

// pub use impl_::*;
