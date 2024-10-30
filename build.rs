//! Build script for ccadical.
//! This script is responsible for compiling the cadical C++ library.
//! For more information:
//! <https://doc.rust-lang.org/cargo/reference/build-scripts.html>
//! <https://doc.rust-lang.org/cargo/reference/build-script-examples.html>

// ************************************************************************************************
// use
// ************************************************************************************************

use std::{
    env::{self},
    // fs,
    path::{Path, PathBuf},
    process::Command,
    thread::available_parallelism,
};

// ************************************************************************************************
// constants
// ************************************************************************************************

// const _CADICAL_PATH: &str = "cadical";

// ************************************************************************************************
// Compile using cc crate
// ************************************************************************************************

// fn _compile_using_cc() {
//     let mut build = cc::Build::new();

//     // set to c++
//     build.cpp(true).flag_if_supported("-std=c++11");

//     // disable default flags
//     // build.no_default_flags(true);

//     // add the flags used by cadical 'configure: compiling with 'g++ -Wall -Wextra -O3 -DNDEBUG -DNBUILD'

//     // this adds -Wall and -Wextra
//     build.warnings(true);

//     // define pre compilation variables
//     build.define("NDEBUG", None);
//     build.define("NBUILD", None);
//     build.define("NUNLOCKED", None);
//     build.define("NTRACING", None);
//     build.define("QUIET", None);

//     let version = std::fs::read_to_string(format!("{_CADICAL_PATH}/VERSION"));
//     let version = version.expect("missing cadical submodule");
//     let version = format!("\"{}\"", version.trim());
//     build.define("VERSION", version.as_ref());

//     // assertions only for debug builds with debug feature enabled
//     if std::env::var("PROFILE").unwrap() == "debug"
//         && std::env::var("CARGO_FEATURE_CPP_DEBUG").is_ok()
//     {
//         build.debug(true);
//     } else {
//         build.debug(false).opt_level(3).define("NDEBUG", None);
//     }

//     // create list of files to compile
//     let mut files = vec![];

//     // add interface that we added
//     // files.push("src/ccadical.cpp".to_string());

//     // add cadical .cpp files
//     let dir_entries = fs::read_dir(format!("{_CADICAL_PATH}/src")).unwrap();
//     for path in dir_entries {
//         let dir_entry = path.unwrap();
//         let path = dir_entry.path();
//         let path_str = path.to_str().unwrap().to_string();
//         if std::path::Path::new(&path_str)
//                      .extension()
//                      .map_or(false, |ext| ext.eq_ignore_ascii_case("cpp"))
//             // mobical should be ignored
//             // && (!path_str.ends_with("/mobical.cpp"))
//             // added later
//             // && (!path_str.ends_with("/resources.cpp"))
//             // added later
//             // && (!path_str.ends_with("/lookahead.cpp"))
//             // already added in src/ccadical.cpp
//             // && (!path_str.ends_with("/ccadical.cpp"))
//             // contains another main function
//             && (!path_str.ends_with("/cadical.cpp"))
//         {
//             // eprintln!("Compiling path {}", path_str);
//             files.push(path_str);
//         }
//     }

//     // add resources and lookahead files
//     // if build.get_compiler().is_like_msvc() {
//     //     build.include(std::path::Path::new("src/msvc"));
//     //     files.push("src/msvc/resources.cpp".to_string());
//     //     files.push("src/msvc/lookahead.cpp".to_string());
//     // } else {
//     //     files.push(format!("{CADICAL_PATH}/src/resources.cpp"));
//     //     files.push(format!("{CADICAL_PATH}/src/lookahead.cpp"));
//     // }

//     // add files which will be compiled
//     build.files(files.iter());

//     // tell the compiler to recompile if any of the files changed
//     for file in &files {
//         println!("cargo:rerun-if-changed={file}");
//     }
//     println!("cargo:rerun-if-env-changed=CC");
//     println!("cargo:rerun-if-env-changed=CFLAGS");
//     println!("cargo:rerun-if-env-changed=CXX");
//     println!("cargo:rerun-if-env-changed=CXXFLAGS");
//     println!("cargo:rerun-if-env-changed=CXXSTDLIB");
//     println!("cargo:rerun-if-env-changed=CRATE_CC_NO_DEFAULTS");

//     // link the standard library if needed (this fixes errors when using Clang)
//     if build.get_compiler().is_like_clang() {
//         build.cpp_set_stdlib("c++");
//     }

//     // compile
//     build.compile("ccadical");
// }

// ************************************************************************************************
// Compile using the ./config && make script
// ************************************************************************************************

// fn _run_command(command: &mut Command) {
//     let command_str = format!("{command:?}");
//     match command.output() {
//         Ok(_) => println!("cargo:warning=Command {command_str} was successful"),
//         Err(e) => {
//             panic!("Failed to execute command:\n{}\nERROR:\n{}", command_str, e);
//         }
//     }
// }

// fn _change_directory(path: &str) {
//     match env::set_current_dir(Path::new(path)) {
//         Ok(()) => println!(
//             "cargo:warning=Changed working directory to {}",
//             env::current_dir().unwrap().display()
//         ),
//         Err(e) => panic!("Failed to change directory to:\n{}\nERROR:\n{}", path, e),
//     }
// }

// fn _make_dir(dir: &str) {
//     match fs::create_dir_all(dir) {
//         Ok(()) => println!("cargo:warning=Created directory {dir}"),
//         Err(e) => panic!("Failed to create directory:\n{}\nERROR:\n{}", dir, e),
//     }
// }

// /// Not ready yet, mainly there are issues with using cargo clean to clean the build.
// /// The problem is that cargo clean will delete the target directory,
// /// which will does not delete the cadical build. Both solutions of either performing
// /// "make clean" on build or making the script compile into target ran into issues.
// fn _compile_using_cadical_script() {
//     // always recompile when anything changes
//     // println!("cargo:rerun-if-changed=/{}", CADICAL_PATH);

//     // change working directory to cadical
//     _change_directory(format!("./{CADICAL_PATH}").as_ref());

//     // clean previous build
//     // _run_command(Command::new("make").arg("clean"));

//     // configure makefile
//     _run_command(&mut Command::new("./configure"));

//     // compile
//     _run_command(&mut Command::new("make"));

//     panic!();
// }

// fn _generate_bindings() {
//     // Tell cargo to look for shared libraries in the specified directory
//     println!(
//         "cargo:rustc-link-search={}/cadical/build",
//         env::var("CARGO_MANIFEST_DIR").unwrap()
//     );

//     // Tell cargo to tell rustc to link the system bzip2
//     // shared library.
//     println!("cargo:rustc-link-lib=cadical");

//     // The bindgen::Builder is the main entry point
//     // to bindgen, and lets you build up options for
//     // the resulting bindings.
//     let bindings = bindgen::Builder::default()
//         // The input header we would like to generate
//         // bindings for.
//         .header("wrapper.h")
//         // Tell cargo to invalidate the built crate whenever any of the
//         // included header files changed.
//         .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
//         // Finish the builder and generate the bindings.
//         .generate()
//         // Unwrap the Result and panic on failure.
//         .expect("Unable to generate bindings for cadical");

//     // Write the bindings to the $OUT_DIR/bindings.rs file.
//     let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
//     bindings
//         .write_to_file(out_path.join("bindings.rs"))
//         .expect("Couldn't write bindings!");
// }

fn compile_using_configuration_script() -> Result<PathBuf, String> {
    let out_dir = env::var("OUT_DIR")
        .map_err(|_| "Environment variable `OUT_DIR` is not defined.".to_string())?;
    let out_dir = Path::new(&out_dir).join("cadical");

    let current_dir = env::var("CARGO_MANIFEST_DIR")
        .map_err(|_| "Environment variable `CARGO_MANIFEST_DIR` is not defined.".to_string())?;
    let current_dir = Path::new(&current_dir);
    let cadical_dir = current_dir.join("cadical");

    let dst = Command::new("./configure")
        .current_dir(&cadical_dir)
        .status()
        .expect("Failed to execute cadical `./configure`.");
    assert!(dst.success(), "Failed to execute cadical `./configure`.");

    let dst = Command::new("make")
        .current_dir(&cadical_dir)
        .arg("-j")
        .arg(available_parallelism().unwrap().get().to_string())
        .status()
        .expect("Failed to execute cadical `make`.");
    assert!(dst.success(), "Failed to execute cadical `make`.");

    // move everything to the output directory
    let dst = Command::new("rm")
        .arg("-rf")
        .arg(&out_dir)
        .status()
        .expect("Failed to execute `rm -rf` while compiling cadical.");
    assert!(
        dst.success(),
        "Failed to execute `mv` while compiling cadical."
    );
    let dst = Command::new("mv")
        .arg("-f")
        .arg(cadical_dir.join("build"))
        .arg(&out_dir)
        .status()
        .expect("Failed to execute `mv` while compiling cadical.");
    assert!(
        dst.success(),
        "Failed to execute `mv` while compiling cadical."
    );

    println!(
        "cargo:rustc-link-lib={}",
        out_dir.join("build").join("libcadical.a").display()
    );
    println!("cargo:rerun-if-env-changed=CC");
    println!("cargo:rerun-if-env-changed=CFLAGS");
    println!("cargo:rerun-if-env-changed=CXX");
    println!("cargo:rerun-if-env-changed=CXXFLAGS");
    println!("cargo:rerun-if-env-changed=CXXSTDLIB");
    println!("cargo:rerun-if-env-changed=CRATE_CC_NO_DEFAULTS");
    println!("cargo:rerun-if-env-changed=cadical/src");

    Ok(out_dir)
}

fn create_bindings() -> Result<(), String> {
    let bindings = autocxx_bindgen::Builder::default()
        .header("wrapper.hpp")
        .enable_cxx_namespaces()
        .allowlist_type("CaDiCaL.*")
        .blocklist_type("std::.*")
        .blocklist_type("__gnu_cxx::new.*")
        // .opaque_type("std::shared_ptr.*")
        // .opaque_type("std::weak_ptr.*")
        // .opaque_type("std::unique_ptr.*")
        .opaque_type("std::vector.*")
        // .opaque_type("std::string.*")
        // .opaque_type("std::optional.*")
        .clang_arg("-fparse-all-comments")
        .parse_callbacks(Box::new(autocxx_bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings for cadical.");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    Ok(())
}

// fn create_bindings_2() -> Result<(), String> {
//     let path = std::path::PathBuf::from("cadical/src"); // include path
//     let mut b = autocxx_build::Builder::new("src/lib.rs", [&path])
//         .build()
//         .map_err(|e| e.to_string())?;
//     b.flag_if_supported("-std=c++11").compile("autocxx-demo"); // arbitrary library name, pick anything
//     println!("cargo:rerun-if-changed=src/lib.rs");
//     Ok(())
// }

// ************************************************************************************************
// Main build function
// ************************************************************************************************

fn main() -> Result<(), String> {
    // print_configurations_for_rust_compiler();
    compile_using_configuration_script()?;
    // _compile_using_cc();
    create_bindings()?;
    Ok(())
}
