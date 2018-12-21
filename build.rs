fn main() {
    if cfg!(not(target_os = "windows")) {
        println!("cargo:rustc-link-lib=dylib=ffi");
    }
}
