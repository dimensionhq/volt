Install `vcpkg`

**Note**: `vcpkg` DOES NOT NEED TO BE IN `PATH`

In `vcpkg` directory:
`./vcpkg.exe integrate install`
`./vcpkg.exe install --recurse curl[http2,openssl]:x64-windows-static-md`

`Cargo.toml`
```toml
isahc = { version = "1.5.0" , features = ["http2", "text-decoding"], default-features = false }
```

`build.rs` (not inside `src`)
```rs
fn main() {
    println!("cargo:rustc-link-lib=nghttp2");
    println!("cargo:rustc-link-lib=libssl");
    println!("cargo:rustc-link-lib=libcrypto");
}
```

`Cargo.toml`
```rs
build = "build.rs"
```

Go to `"C:\Users\xtrem\.cargo\registry\src\github.com-1ecc6299db9ec823\curl-sys-0.4.49+curl-7.79.1\build.rs"`

Comment out the following `if` statement
```rs
    // if !Path::new("curl/.git").exists() {
    //     let _ = Command::new("git")
    //         .args(&["submodule", "update", "--init"])
    //         .status();
    // }
```

`cargo clean`

request code:
```rs
let client: Request<&str> =
    Request::get(format!("https://registry.npmjs.org/{}", package_name))
        .header(
            "accept",
            "application/vnd.npm.install-v1+json; q=1.0, application/json; q=0.8, */*",
        )
        .header("accept-encoding", "gzip,deflate")
        .header("connection", "keep-alive")
        .header("host", "registry.npmjs.org")
        .version_negotiation(VersionNegotiation::http2())
        .ssl_options(SslOption::DANGER_ACCEPT_INVALID_CERTS)
        .body("")
        .map_err(VoltError::RequestBuilderError)?;
```

`cargo build --release`