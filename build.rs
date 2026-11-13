use std::env;

fn main() {
    const BACKEND: &str = "BRZ_COLLECTIONS_BACKEND";

    println!("cargo:rerun-if-env-changed={BACKEND}");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rustc-check-cfg=cfg(brz_cpython_set)");

    let feature_enabled = env::var_os("CARGO_FEATURE_CPYTHON_SET").is_some();
    let cpython = match env::var(BACKEND) {
        Ok(value) if value == "cpython" => true,
        Ok(value) if value == "rust" && !feature_enabled => false,
        Ok(value) if value == "rust" => panic!("{BACKEND}=rust conflicts with cpython-set"),
        Ok(value) => panic!("invalid {BACKEND}={value:?}; expected rust or cpython"),
        Err(env::VarError::NotPresent) => feature_enabled,
        Err(err) => panic!("could not read {BACKEND}: {err}"),
    };

    if cpython {
        println!("cargo:rustc-cfg=brz_cpython_set");
    }
}
