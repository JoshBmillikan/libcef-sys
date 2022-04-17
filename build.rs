use std::env;
use std::error::Error;
use std::fs::read_dir;
use std::path::PathBuf;

fn main() {
    let (mut include, lib) = find_cef().expect(
        "Could not find cef, please install the cef library. \
     (Ensure CEF_ROOT is set to the path to the library, or that it is available to pkg_config)",
    );
    println!("cargo:rustc-link-search=native={}", lib.to_str().unwrap());
    println!("cargo:rustc-link-lib=libcef");
    let c_headers = include.join("capi");
    let mut bindings = bindgen::Builder::default()
        .clang_arg(format!("-I{}", include.to_str().unwrap()))
        .rustfmt_bindings(true)
        .blocklist_file("Windows.h")
        .opaque_type("_IMAGE_.*")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks));

    for file in read_dir(c_headers.clone()).expect(
        format!(
            "Could not open include dir: {}",
            c_headers.to_str().unwrap()
        )
        .as_str(),
    ) {
        if let Ok(file) = file {
            if !file.file_type().unwrap().is_dir() {
                let path = file.path();
                bindings = bindings.header(path.to_str().unwrap());
            }
        }
    }
    include.pop();
    let bindings = bindings
        .clang_arg(format!("-I{}", include.to_str().unwrap()))
        .generate()
        .expect("Failed to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

fn find_cef() -> Result<(PathBuf, PathBuf), Box<dyn Error>> {
    if let Ok(path) = env::var("CEF_ROOT") {
        let root = PathBuf::from(path);
        Ok((root.join("include"), root.join("Release")))
    } else {
        let lib = pkg_config::Config::new().probe("cef")?;
        Ok((lib.include_paths[0].clone(), lib.link_paths[0].clone()))
    }
}
