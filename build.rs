use fs_extra::dir::CopyOptions;
use std::default::Default;
use std::env;
use std::error::Error;
use std::fs::{copy, create_dir, read_dir};
use std::path::PathBuf;

fn main() {
    let (mut include, lib) = find_cef().expect(
        "Could not find cef, please install the cef library. \
     (Ensure CEF_ROOT is set to the path to the library, or that it is available to pkg_config)",
    );
    println!("cargo:rustc-link-search=native={}", lib.to_str().unwrap());
    println!("cargo:rustc-link-lib=libcef");
    let c_headers = include.join("capi");
    let bindings = bindgen::Builder::default()
        .clang_arg(format!("-I{}", include.to_str().unwrap()))
        .clang_arg("-D WIN32_LEAN_AND_MEAN")
        .rustfmt_bindings(true);
    let mut bindings = if cfg!(target_os = "windows") {
        // because windows....
        bindings
            .blocklist_file("windows.h")
            .blocklist_file("Windows.h")
            .blocklist_file("win.*")
            .blocklist_type("LPMONITORINFOEXA?W?")
            .blocklist_type("LPTOP_LEVEL_EXCEPTION_FILTER")
            .blocklist_type("MONITORINFOEXA?W?")
            .blocklist_type("PEXCEPTION_FILTER")
            .blocklist_type("PEXCEPTION_ROUTINE")
            .blocklist_type("PSLIST_HEADER")
            .blocklist_type("PTOP_LEVEL_EXCEPTION_FILTER")
            .blocklist_type("PVECTORED_EXCEPTION_HANDLER")
            .blocklist_type("_?L?P?CONTEXT")
            .blocklist_type("_?L?P?EXCEPTION_POINTERS")
            .blocklist_type("_?P?DISPATCHER_CONTEXT")
            .blocklist_type("_?P?EXCEPTION_REGISTRATION_RECORD")
            .blocklist_type("_?P?IMAGE_TLS_DIRECTORY.*")
            .blocklist_type("_?P?NT_TIB")
            .blocklist_type("tagMONITORINFOEXA")
            .blocklist_type("tagMONITORINFOEXW")
            .blocklist_function("AddVectoredContinueHandler")
            .blocklist_function("AddVectoredExceptionHandler")
            .blocklist_function("CopyContext")
            .blocklist_function("GetThreadContext")
            .blocklist_function("GetXStateFeaturesMask")
            .blocklist_function("InitializeContext")
            .blocklist_function("InitializeContext2")
            .blocklist_function("InitializeSListHead")
            .blocklist_function("InterlockedFlushSList")
            .blocklist_function("InterlockedPopEntrySList")
            .blocklist_function("InterlockedPushEntrySList")
            .blocklist_function("InterlockedPushListSListEx")
            .blocklist_function("LocateXStateFeature")
            .blocklist_function("QueryDepthSList")
            .blocklist_function("RaiseFailFastException")
            .blocklist_function("RtlCaptureContext")
            .blocklist_function("RtlCaptureContext2")
            .blocklist_function("RtlFirstEntrySList")
            .blocklist_function("RtlInitializeSListHead")
            .blocklist_function("RtlInterlockedFlushSList")
            .blocklist_function("RtlInterlockedPopEntrySList")
            .blocklist_function("RtlInterlockedPushEntrySList")
            .blocklist_function("RtlInterlockedPushListSListEx")
            .blocklist_function("RtlQueryDepthSList")
            .blocklist_function("RtlRestoreContext")
            .blocklist_function("RtlUnwindEx")
            .blocklist_function("RtlVirtualUnwind")
            .blocklist_function("SetThreadContext")
            .blocklist_function("SetUnhandledExceptionFilter")
            .blocklist_function("SetXStateFeaturesMask")
            .blocklist_function("UnhandledExceptionFilter")
            .blocklist_function("__C_specific_handler")
    } else {
        bindings
    }
    .derive_default(true)
    .size_t_is_usize(true)
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

    copy_libs(lib);
}

fn copy_libs(mut path: PathBuf) {
    let out = find_cargo_target_dir();
    let regex = regex::Regex::new(".*\\.dll|.*\\.so|.*\\.dylib").unwrap();
    for file in read_dir(&path).expect("Could not read shared libraries") {
        if let Ok(file) = file {
            if regex.is_match(file.file_name().to_str().unwrap()) {
                copy(file.path(), &out.join(file.file_name())).expect(&format!(
                    "Failed to copy dynamic library {} to {}",
                    file.path().to_string_lossy(),
                    out.to_string_lossy()
                ));
                println!(
                    "Copied {} to target directory",
                    file.path().to_string_lossy()
                );
            }
        }
    }
    path.pop();
    let resources = path.join("Resources");
    println!("{}",resources.to_string_lossy());
    let mut options = CopyOptions::new();
    options.overwrite = true;
    options.copy_inside = true;
    let _ = fs_extra::dir::copy(&resources, &out, &options)
        .expect("Failed to copy resources");
}

// borrowed from Rust-SDL2's build script
fn find_cargo_target_dir() -> PathBuf {
    // Infer the top level cargo target dir from the OUT_DIR by searching
    // upwards until we get to $CARGO_TARGET_DIR/build/ (which is always one
    // level up from the deepest directory containing our package name)
    let pkg_name = env::var("CARGO_PKG_NAME").unwrap();
    let mut out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    loop {
        {
            let final_path_segment = out_dir.file_name().unwrap();
            if final_path_segment.to_string_lossy().contains(&pkg_name) {
                break;
            }
        }
        if !out_dir.pop() {
            panic!("Malformed build path: {}", out_dir.to_string_lossy());
        }
    }
    out_dir.pop();
    out_dir.pop();
    out_dir
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
