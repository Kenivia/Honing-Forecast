mod v1;
mod v10;
mod v11;
mod v12;
mod v13;
mod v14;
mod v2;
mod v3;
mod v4;
mod v5;
mod v6;
mod v7;
mod v8;
mod v9;
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
    }    else if #[cfg(feature = "v7")] {
        pub use v7::*;
        pub static ACTIVE_FEATURE :&str= "v7";
    }
     else if #[cfg(feature = "v8")] {
        pub use v8::*;
        pub static ACTIVE_FEATURE :&str= "v8";
    }
         else if #[cfg(feature = "v9")] {
        pub use v9::*;
        pub static ACTIVE_FEATURE :&str= "v9";
    }
             else if #[cfg(feature = "v10")] {
        pub use v10::*;
        pub static ACTIVE_FEATURE :&str= "v10";
    }       else if #[cfg(feature = "v11")] {
        pub use v11::*;
        pub static ACTIVE_FEATURE :&str= "v11";
    }           else if #[cfg(feature = "v12")] {
        pub use v12::*;
        pub static ACTIVE_FEATURE :&str= "v12";
    }  else if #[cfg(feature = "v13")] {
        pub use v13::*;
        pub static ACTIVE_FEATURE :&str= "v13";
    }
    else if #[cfg(feature = "v14")] {
        pub use v14::*;
        pub static ACTIVE_FEATURE :&str= "v14";
    }
}
