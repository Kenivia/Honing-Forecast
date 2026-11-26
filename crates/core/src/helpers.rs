//! `my_dbg!`: prints to the browser console in wasm and to stderr natively.

#[doc(hidden)]
pub fn __log(msg: &str) {
    #[cfg(target_arch = "wasm32")]
    web_sys::console::log_1(&msg.into());
    #[cfg(not(target_arch = "wasm32"))]
    eprintln!("{msg}");
}

#[macro_export]
macro_rules! my_dbg {
    () => {
        $crate::helpers::__log(&format!("[{}:{}]", file!(), line!()))
    };
    ($($val:expr),+ $(,)?) => {{
        $(
            $crate::helpers::__log(&format!(
                "[{}:{}] {} = {:#?}",
                file!(),
                line!(),
                stringify!($val),
                &$val
            ));
        )+
    }};
}
