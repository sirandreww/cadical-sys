use std::process::Command;

fn init_submodules() {
    if std::path::Path::new("cadical/src").exists() {
        return;
    }
    let output = Command::new("git")
        .args(["submodule", "update", "--init"])
        .output()
        .expect("cadical submodule is missing and `git` could not be run to fetch it");
    assert!(
        output.status.success(),
        "cadical submodule is missing and `git submodule update --init` failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
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

    build.define("NCLOSEFROM", None);

    let version = std::fs::read_to_string(format!("{CADICAL_PATH}/VERSION"))
        .expect("missing cadical submodule");
    let version = format!("\"{}\"", version.trim());
    build.define("VERSION", version.as_ref());

    build.debug(false).opt_level(3);

    // Like CaDiCaL's makefile: every source except the two applications.
    let mut files = vec![];
    let mut c_files = vec![];
    let dir_entries = std::fs::read_dir(format!("{CADICAL_PATH}/src")).unwrap();
    for path in dir_entries {
        let path = path.unwrap().path();
        let is_app = path
            .file_name()
            .is_some_and(|name| name == "cadical.cpp" || name == "mobical.cpp");
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("cpp") if !is_app => files.push(path),
            Some("c") => c_files.push(path),
            _ => {}
        }
    }
    build.files(files.iter());

    // Rerun if these change
    for file in files.iter().chain(&c_files) {
        println!("cargo:rerun-if-changed={}", file.display());
    }
    println!("cargo:rerun-if-changed=src/cadical_bridge.hpp");
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=src/bridge.rs");
    println!("cargo:rerun-if-changed=build.rs");
    // Headers and the version, which is compiled in as VERSION.
    println!("cargo:rerun-if-changed={CADICAL_PATH}/src");
    println!("cargo:rerun-if-changed={CADICAL_PATH}/VERSION");

    // Standard build environment vars
    println!("cargo:rerun-if-env-changed=CC");
    println!("cargo:rerun-if-env-changed=CFLAGS");
    println!("cargo:rerun-if-env-changed=CXX");
    println!("cargo:rerun-if-env-changed=CXXFLAGS");
    println!("cargo:rerun-if-env-changed=CXXSTDLIB");
    println!("cargo:rerun-if-env-changed=CRATE_CC_NO_DEFAULTS");

    // Use the compiler's default C++ standard library, like cxx does for its
    // own runtime: mixing libc++ and libstdc++ fails to link with clang on Linux.
    // Compile cxx bridge
    build.compile("cadical-rs-bridge");

    // CaDiCaL 3 also has C sources ('kitten.c'). They are compiled after the
    // C++ library so that they come later in the link order.
    cc::Build::new()
        .files(c_files.iter())
        .define("NDEBUG", None)
        .opt_level(3)
        .compile("cadical-c");
}

fn main() {
    init_submodules();
    compile_cadical();
}
