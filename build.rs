use std::{env, path::PathBuf, process::Command};

fn main() {
    println!("cargo:rerun-if-changed=assets/app.rc");
    println!("cargo:rerun-if-changed=assets/app.ico");
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let gnu = env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("gnu");
    let output = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let resource = output.join("app.res");
    let mut compiler = Command::new(if gnu { "llvm-rc" } else { "rc" });
    if gnu {
        compiler.arg("/no-preprocess");
    } else {
        compiler.arg("/nologo");
    }
    let status = compiler
        .arg("/fo")
        .arg(&resource)
        .arg("assets/app.rc")
        .status()
        .expect("Windows resource compiler missing; use scripts/Build.ps1 -Bootstrap");
    assert!(status.success(), "Windows icon resource compilation failed");
    let linked = if gnu {
        let object = output.join("app.o");
        let machine = match env::var("CARGO_CFG_TARGET_ARCH").unwrap().as_str() {
            "x86_64" => "/machine:x64",
            "x86" => "/machine:x86",
            "aarch64" => "/machine:arm64",
            _ => panic!("Unsupported Windows resource architecture"),
        };
        let status = Command::new("llvm-cvtres")
            .args([machine, "/timestamp:0"])
            .arg(format!("/out:{}", object.display()))
            .arg(&resource)
            .status()
            .expect("llvm-cvtres missing; use scripts/Build.ps1 -Bootstrap");
        assert!(status.success(), "Windows icon resource conversion failed");
        object
    } else {
        resource
    };
    println!("cargo:rustc-link-arg={}", linked.display());
}
