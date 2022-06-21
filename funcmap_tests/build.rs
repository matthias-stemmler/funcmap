use std::process::{Command, ExitStatus, Stdio};

fn has_cargo_expand() -> bool {
    let cargo_expand = if cfg!(windows) {
        "cargo-expand.exe"
    } else {
        "cargo-expand"
    };

    Command::new(cargo_expand)
        .arg("--version")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .as_ref()
        .map(ExitStatus::success)
        .unwrap_or(false)
}

fn has_rustfmt() -> bool {
    toolchain_find::find_installed_component("rustfmt").is_some()
}

fn is_minimal_versions() -> bool {
    option_env!("FUNCMAP_MINIMAL_VERSIONS").is_some()
}

fn main() {
    if has_cargo_expand() {
        println!("cargo:rustc-cfg=has_cargo_expand");
    }

    if has_rustfmt() {
        println!("cargo:rustc-cfg=has_rustfmt");
    }

    if is_minimal_versions() {
        println!("cargo:rustc-cfg=is_minimal_versions");
    }
}
