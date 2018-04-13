fn main() {
    #[cfg(not(any(feature = "llvm3-6", feature = "llvm3-7")))]
    println!("cargo:rustc-link-lib=dylib=ffi");
}
