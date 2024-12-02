use std::{path::Path, process::Command};

fn init_submodules() {
    if std::path::Path::new("cadical/src").exists() {
        return;
    }
    let _ = Command::new("git")
        .arg("submodule")
        .arg("update")
        .arg("--init")
        .output()
        .expect("Failed to execute git submodule update --init.");
}

fn compile_cadical() {
    const CADICAL_PATH: &str = "cadical";
    let mut build = cxx_build::bridge("src/bridge.rs");

    build.cpp(true).flag_if_supported("-std=c++11");

    build.warnings(true);

    build.define("NDEBUG", None);
    build.define("NBUILD", None);
    build.define("NUNLOCKED", None);
    build.define("NTRACING", None);
    build.define("QUIET", None);

    let version = std::fs::read_to_string(format!("{CADICAL_PATH}/VERSION"))
        .expect("missing cadical submodule");
    let version = format!("\"{}\"", version.trim());
    build.define("VERSION", version.as_ref());

    if std::env::var("PROFILE").unwrap() == "debug"
        && std::env::var("CARGO_FEATURE_CPP_DEBUG").is_ok()
    {
        build.debug(true);
    } else {
        build.debug(false).opt_level(3).define("NDEBUG", None);
    }

    let mut files = vec![];

    let dir_entries = std::fs::read_dir(format!("{CADICAL_PATH}/src")).unwrap();
    for path in dir_entries {
        let dir_entry = path.unwrap();
        let path = dir_entry.path();
        let path_str = path.to_str().unwrap().to_string();
        if Path::new(&path_str)
            .extension()
            .map_or(false, |ext| ext.eq_ignore_ascii_case("cpp"))
            && (!path_str.ends_with("/cadical.cpp"))
        {
            files.push(path_str);
        }
    }
    // files.push("src/cadical_bridge.cpp".to_string());

    // Add bridge source for cxx
    // files.push("src/cadical_bridge.cpp".to_string());

    build.files(files.iter());

    // Rerun if these change
    for file in &files {
        println!("cargo:rerun-if-changed={file}");
    }
    println!("cargo:rerun-if-changed=src/cadical_bridge.hpp");
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=src/bridge.rs");
    println!("cargo:rerun-if-changed=build.rs");

    // Standard build environment vars
    println!("cargo:rerun-if-env-changed=CC");
    println!("cargo:rerun-if-env-changed=CFLAGS");
    println!("cargo:rerun-if-env-changed=CXX");
    println!("cargo:rerun-if-env-changed=CXXFLAGS");
    println!("cargo:rerun-if-env-changed=CXXSTDLIB");
    println!("cargo:rerun-if-env-changed=CRATE_CC_NO_DEFAULTS");

    if build.get_compiler().is_like_clang() {
        build.cpp_set_stdlib("c++");
    }

    // Compile cxx bridge
    build.compile("cadical-rs-bridge");
}

fn main() {
    init_submodules();
    compile_cadical();
}
