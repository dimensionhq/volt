fn main() {
    if cfg!(windows) {
        println!("cargo:rustc-link-lib=nghttp2");
        println!("cargo:rustc-link-lib=libssl");
        println!("cargo:rustc-link-lib=libcrypto");
    } else {
        println!("cargo:rustc-link-lib=nghttp2");
        println!("cargo:rustc-link-lib=ssl");
        println!("cargo:rustc-link-lib=crypto");
    };
}