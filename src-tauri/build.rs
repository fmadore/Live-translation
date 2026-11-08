// The command list is `include!`d rather than duplicated: `AppManifest::commands` generates an
// `allow-<command>` permission for each name, and the same list is compiled into the crate so a
// test can check it against the handler and the capability files. See `src/command_names.rs`.
include!("src/command_names.rs");

fn main() {
    link_clang_runtime_for_metal();
    tauri_build::try_build(
        tauri_build::Attributes::new()
            .app_manifest(tauri_build::AppManifest::new().commands(COMMANDS)),
    )
    // Declaring an app manifest is what makes Tauri enforce the ACL on the app's own commands,
    // so a failure here would silently mean an unenforced split rather than a broken build.
    .expect("failed to generate the app ACL manifest");
}

/// On macOS Whisper is built with Metal (`Cargo.toml`), whose code checks the system's version
/// at run time (`@available`). That compiles to a call to `__isPlatformVersionAtLeast` in
/// clang's own runtime library, which Rust does not link, so the app would fail to link; it is
/// added here, from the clang that built whisper.cpp. The same fix as Kalamu's
/// `crates/offline/build.rs`.
fn link_clang_runtime_for_metal() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("macos") {
        return;
    }
    let resources = std::process::Command::new("xcrun")
        .args(["clang", "--print-resource-dir"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| std::path::PathBuf::from(String::from_utf8_lossy(&output.stdout).trim()));
    match resources {
        Some(dir) => {
            println!(
                "cargo:rustc-link-search=native={}",
                dir.join("lib").join("darwin").display()
            );
            println!("cargo:rustc-link-lib=static:-bundle=clang_rt.osx");
        }
        None => println!(
            "cargo:warning=clang's runtime library was not found: linking Whisper's Metal code may fail"
        ),
    }
}
