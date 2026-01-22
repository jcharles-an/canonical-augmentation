use std::{env, fs};
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let n_value = std::env::var("N_C").unwrap_or("7".to_string());
    let n_method = std::env::var("N_METHOD").unwrap_or("1".to_string());

    let dest_path = Path::new(&out_dir).join("consts.rs");
    fs::write(
        &dest_path,
        format!("pub const N_C: usize = {}; \n pub const N_METHOD: usize = {};", n_value, n_method),
    ).unwrap();

    println!("cargo:rerun-if-env-changed=N_C");
    println!("cargo:rerun-if-env-changed=N_METHOD");
}