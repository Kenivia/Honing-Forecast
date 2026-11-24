// The exes load their own DLL imports (vcruntime140, bcrypt) from System32 only, never from beside the exe.
fn main() {
    if std::env::var("CARGO_CFG_TARGET_ENV").unwrap() == "msvc" {
        println!("cargo:rustc-link-arg-bins=/DEPENDENTLOADFLAG:0x800");
    }
}
