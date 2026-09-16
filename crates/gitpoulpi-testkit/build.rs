fn main() {
    // The fixture builder links libgit2 directly, so it needs the same
    // advapi32 link directive as gitpoulpi-git2-backend's build.rs: without
    // it the MSVC link of the testkit's own test binary fails with an
    // unresolved GetNamedSecurityInfoW.
    if cfg!(windows) {
        println!("cargo:rustc-link-lib=dylib=advapi32");
    }
}
