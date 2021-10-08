fn main() {
    println!("cargo:rustc-link-lib=nghttp2");
    println!("cargo:rustc-link-lib=libssl");
    println!("cargo:rustc-link-lib=libcrypto");
}
