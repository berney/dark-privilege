use std::process::Command;
use std::env;


fn embed_git_info() {
    // Run `git describe` to get the current version information
    // this will be something like `heads/main-0-gb53ec29-dirty`
    let git_version = Command::new("git")
        .args(&["describe", "--all", "--long", "--dirty"])
        .output()
        .expect("Failed to execute git")
        .stdout;

    // Convert to string and trim
    let git_version_str = String::from_utf8(git_version)
        .expect("Git output not UTF-8")
        .trim()
        .to_string();

    // Add git version as an environment variable for the Rust code
    println!("cargo:rustc-env=GIT_VERSION={}", git_version_str);
}


fn what_c_linkage() {
    let linkage = env::var("CARGO_CFG_TARGET_FEATURE").unwrap_or(String::new());

    if linkage.contains("crt-static") {
        eprintln!("the C runtime will be statically linked");
    } else {
        eprintln!("the C runtime will be dynamically linked");
    }
}


fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    embed_git_info();
    what_c_linkage();

    let exe_name = env::var("CARGO_BIN_NAME").unwrap_or_else(|_| "unknown".to_string());
    eprintln!("XXX exe_name {exe_name}");
    // If we're building the import binary, link the DLL
    if exe_name == "dark-privilege-import" {
        // Tell cargo to link against the DLL, using the import table
        //println!("cargo:rustc-link-lib=dylib=dark_privilege_lib");
        println!("cargo:rustc-link-lib=cdylib=dark_privilege_lib");
        println!("cargo:rustc-link-search=native=C:/Users/berne/dark-privilege/target/x86_64-pc-windows-msvc/debug");
        //println!("cargo:rustc-link-search=native=xxxyyyzzzXXXYYYZZZxxxyyyzzzXXXYYYZZZ"); // Replace with the actual DLL path
    }

    if env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("dark-privilege.ico");
        // This should ended up being embedded inside the exe
        res.set_manifest_file("dark-privilege.exe.manifest");
        res.compile().unwrap();
    }
}
