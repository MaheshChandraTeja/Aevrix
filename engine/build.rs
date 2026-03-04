fn main() {
    println!("cargo:rerun-if-env-changed=AEVRIX_BUILD_VERSION");

    let ver = std::env::var("AEVRIX_BUILD_VERSION").unwrap_or_else(|_| "dev".into());
    println!("cargo:rustc-env=AEVRIX_BUILD_VERSION={}", ver);

    println!("cargo:rerun-if-changed=Cargo.toml");
}
