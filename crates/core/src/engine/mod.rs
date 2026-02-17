//! If there was a better way to do this, I couldn't be bothered to find it.
//! This just routes which engine to use with --feature
//! Current in-use version is v35 at the time of writing this
//! note that previous versions are unlikely to work out of the box because a lot of stuff have changed

cfg_if::cfg_if! {
    if #[cfg(feature = "v1")] {
        mod v1;
        pub use v1::*;
        pub const ACTIVE_FEATURE :&str= "v1";
    } else if #[cfg(feature = "v2")] {
        mod v2;
        pub use v2::*;
        pub const ACTIVE_FEATURE :&str= "v2";
    }else if #[cfg(feature = "v3")] {
            mod v3;
        pub use v3::*;
        pub const ACTIVE_FEATURE :&str= "v3";
    }else if #[cfg(feature = "v4")] {
            mod v4;
        pub use v4::*;
        pub const ACTIVE_FEATURE :&str= "v4";
    }else if #[cfg(feature = "v5")] {
            mod v5;
        pub use v5::*;
        pub const ACTIVE_FEATURE :&str= "v5";
    }
    else if #[cfg(feature = "v6")] {
            mod v6;
        pub use v6::*;
        pub const ACTIVE_FEATURE :&str= "v6";
    }    else if #[cfg(feature = "v7")] {
            mod v7;
        pub use v7::*;
        pub const ACTIVE_FEATURE :&str= "v7";
    }
     else if #[cfg(feature = "v8")] {
            mod v8;
        pub use v8::*;
        pub const ACTIVE_FEATURE :&str= "v8";
    }
         else if #[cfg(feature = "v9")] {
                mod v9;
        pub use v9::*;
        pub const ACTIVE_FEATURE :&str= "v9";
    }
             else if #[cfg(feature = "v10")] {
                    mod v10;
        pub use v10::*;
        pub const ACTIVE_FEATURE :&str= "v10";
    }       else if #[cfg(feature = "v11")] {
            mod v11;
        pub use v11::*;
        pub const ACTIVE_FEATURE :&str= "v11";
    }           else if #[cfg(feature = "v12")] {
            mod v12;
        pub use v12::*;
        pub const ACTIVE_FEATURE :&str= "v12";
    }  else if #[cfg(feature = "v13")] {
            mod v13;
        pub use v13::*;
        pub const ACTIVE_FEATURE :&str= "v13";
    }
    else if #[cfg(feature = "v14")] {
            mod v14;
        pub use v14::*;
        pub const ACTIVE_FEATURE :&str= "v14";
    }
       else if #[cfg(feature = "v15")] {
            mod v15;
        pub use v15::*;
        pub const ACTIVE_FEATURE :&str= "v15";
    }
           else if #[cfg(feature = "v16")] {
                mod v16;
        pub use v16::*;
        pub const ACTIVE_FEATURE :&str= "v16";
    }
               else if #[cfg(feature = "v17")] {
                    mod v17;
        pub use v17::*;
        pub const ACTIVE_FEATURE :&str= "v17";
    }
               else if #[cfg(feature = "v18")] {
                    mod v18;
        pub use v18::*;
        pub const ACTIVE_FEATURE :&str= "v18";
    }
               else if #[cfg(feature = "v19")] {
                    mod v19;
        pub use v19::*;
        pub const ACTIVE_FEATURE :&str= "v19";
    }
               else if #[cfg(feature = "v20")] {
                    mod v20;
        pub use v20::*;
        pub const ACTIVE_FEATURE :&str= "v20";
    }

               else if #[cfg(feature = "v21")] {
                      mod v21;
        pub use v21::*;
        pub const ACTIVE_FEATURE :&str= "v21";
    }
               else if #[cfg(feature = "v22")] {
                      mod v22;
        pub use v22::*;
        pub const ACTIVE_FEATURE :&str= "v22";
    }
               else if #[cfg(feature = "v23")] {
                      mod v23;
        pub use v23::*;
        pub const ACTIVE_FEATURE :&str= "v23";
    }
    else if #[cfg(feature = "v24")] {
              mod v24;
        pub use v24::*;
        pub const ACTIVE_FEATURE :&str= "v24";
    }
    else if #[cfg(feature = "v25")] {
              mod v25;
        pub use v25::*;
        pub const ACTIVE_FEATURE :&str= "v25";
    }
    else if #[cfg(feature = "v26")] {
              mod v26;
        pub use v26::*;
        pub const ACTIVE_FEATURE :&str= "v26";
    }
    else if #[cfg(feature = "v27")] {
              mod v27;
        pub use v27::*;
        pub const ACTIVE_FEATURE :&str= "v27";
    }

    else if #[cfg(feature = "v28")] {
              mod v28;
        pub use v28::*;
        pub const ACTIVE_FEATURE :&str= "v28";
    }
    else if #[cfg(feature = "v29")] {
              mod v29;
        pub use v29::*;
        pub const ACTIVE_FEATURE :&str= "v29";
    }
    else if #[cfg(feature = "v30")] {
              mod v30;
        pub use v30::*;
        pub const ACTIVE_FEATURE :&str= "v30";
    }

        else if #[cfg(feature = "v31")] {
             mod v31;
        pub use v31::*;
        pub const ACTIVE_FEATURE :&str= "v31";
    }
            else if #[cfg(feature = "v32")] {
                 mod v32;
        pub use v32::*;
        pub const ACTIVE_FEATURE :&str= "v32";
    }

else if #[cfg(feature = "v33")] {
     mod v33;
        pub use v33::*;
        pub const ACTIVE_FEATURE :&str= "v33";
    }
    else if #[cfg(feature = "v34")] {
         mod v34;
pub use v34::*;
pub const ACTIVE_FEATURE :&str= "v34";
}
else if #[cfg(feature = "v35")] {
    mod v35;
pub use v35::*;
pub const ACTIVE_FEATURE :&str= "v35";
}
}
