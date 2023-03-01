use std::fs::{create_dir, remove_dir_all};
use std::path::Path;

fn main() {
    let out_dir_string = std::env::var("OUT_DIR").unwrap();
    let out_dir = Path::new(out_dir_string.as_str());
    assert!(out_dir.exists());
    assert!(out_dir.is_dir());
    println!("cargo:rustc-env=MACRO_OUT_DIR={}", out_dir.display());
    let target_dir = out_dir.parent().unwrap().parent().unwrap();
    assert!(target_dir.exists());
    let refs_dir = target_dir.join(Path::new("__export_tokens_refs"));
    if refs_dir.exists() {
        remove_dir_all(&refs_dir).unwrap();
    }
    create_dir(&refs_dir).unwrap();
    assert!(refs_dir.exists());
    assert!(refs_dir.is_dir());
    println!("cargo:rustc-env=REFS_DIR={}", refs_dir.display());
}
