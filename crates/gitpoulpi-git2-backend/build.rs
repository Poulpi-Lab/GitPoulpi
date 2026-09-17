fn main() {
    // libgit2-sys's Windows build uses CryptoAPI (hashing/RNG) and the
    // registry/security APIs, but doesn't always emit the advapi32 link
    // directive itself on MSVC -- without this the final link fails with
    // unresolved externals like CryptAcquireContextA / RegOpenKeyExW.
    if cfg!(windows) {
        println!("cargo:rustc-link-lib=dylib=advapi32");
    }
}
