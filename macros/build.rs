fn main() {
    println!(
        "cargo:rustc-env=MACRO_OUT_DIR={}",
        std::env::var("OUT_DIR").unwrap()
    )
}
