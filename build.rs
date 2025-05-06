#![feature(exit_status_error)]

use std::env;
use std::fs;
use std::path::{PathBuf, Path};

fn main() -> Result<(), String> {
    giputils::build::git_submodule_update()?;

    println!("cargo:rerun-if-changed=./bindings");
    println!("cargo:rerun-if-changed=./cadical");
    let cadical_cpps = collect_cpps(&[&Path::new("cadical/src")]);
    let mut build = cc::Build::new();
    build.files(cadical_cpps)
        .file("cadical/contrib/craigtracer.cpp")
        .file("bindings/binding.cpp")
        .includes([
            "cadical/src",
            "cadical/contrib",
            "bindings",
            "/usr/include/c++/v1",
            "/usr/include/",
        ])
        .no_default_flags(true)
        .define("NCLOSEFROM", "1")
        .cpp(true)
        .cpp_link_stdlib(None)
        .warnings(false)
        .opt_level(3)
        .std("c++17")
        .static_flag(true)
        .shared_flag(false)
        .flag("-nostdinc")
        .flag("-nostdlib")
        .flag("-Wno-shift-op-parentheses");
    if let Ok(target) = std::env::var("TARGET") {
        if target.contains("musl") {
            if let Ok(wrapper) = std::env::var("RUST_WRAPPER") {
                if wrapper.contains("sccache") {
                    unsafe {std::env::set_var("CXX", "sccache clang++")};
                }
            }
        }
    }
    build.compile("cadical");

    let out_dir = &env::var("OUT_DIR").unwrap();
    let out_dir = Path::new(out_dir);
    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-link-lib=static=cadical");
    Ok(())
}

fn collect_cpps(verific_dirs: &[&Path]) -> Vec<PathBuf> {
    let mut cpps = vec![];
    for dir in verific_dirs.iter() {
        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let p = entry.path().canonicalize().unwrap();
            match p.extension().map(|x| x.to_str().unwrap()) {
                Some("cpp") => {
                    cpps.push(p);
                }
                _ => {}
            }
        }
    }
    cpps
}
