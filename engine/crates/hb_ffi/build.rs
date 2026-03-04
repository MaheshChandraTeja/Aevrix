fn main() {
    
    println!("cargo:rerun-if-changed=include/hb_ffi.h");
    
    if let Ok(ver) = std::env::var("AEVRIX_BUILD_VERSION") {
        println!("cargo:rustc-env=AEVRIX_BUILD_VERSION={}", ver);
    } else {
        println!("cargo:rustc-env=AEVRIX_BUILD_VERSION=dev");
    }
}
