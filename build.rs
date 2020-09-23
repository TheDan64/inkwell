fn main() {
    if cfg!(all(not(target_os = "windows"), not(feature = "no-libffi-linking"))) {
        println!("cargo:rustc-link-lib=dylib=ffi");
    }
}
