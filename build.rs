use std::{
    env,
    path::{Path, PathBuf},
};

fn run_pkg_config() -> Option<Vec<String>> {
    use std::collections::HashSet;

    let mut pkg = pkg_config::Config::new();
    pkg.cargo_metadata(true)
        .atleast_version("2")
        .statik(cfg!(feature = "static"));

    let mut include_paths = HashSet::new();

    if let Ok(mut lib) = pkg
        .probe("libevent_core")
        .or_else(|_| pkg.probe("libevent"))
    {
        include_paths.extend(lib.include_paths.drain(..));
    } else {
        return None;
    }

    {
        match pkg.probe("libevent_extra") {
            Ok(mut lib) => include_paths.extend(lib.include_paths.drain(..)),
            Err(e) => println!("Failed to find libevent_extra: {:?}", e),
        }
    }

    if cfg!(feature = "openssl") {
        let mut lib = pkg.cargo_metadata(true).probe("libevent_openssl").unwrap();
        include_paths.extend(lib.include_paths.drain(..));
    }

    if cfg!(feature = "threading") {
        let mut lib = pkg.cargo_metadata(true).probe("libevent_pthreads").unwrap();
        include_paths.extend(lib.include_paths.drain(..));
    }

    let include_paths = include_paths
        .drain()
        .map(|path| {
            let path_s = path.into_os_string().into_string().unwrap();
            println!("cargo:include={}", &path_s);
            path_s
        })
        .collect();

    Some(include_paths)
}

fn generate_bindings(include_paths: Vec<String>, out_path: impl AsRef<Path>) {
    println!("cargo:rerun-if-changed=libevent");
    println!("cargo:rerun-if-changed=wrapper.h");

    let mut builder = bindgen::Builder::default()
        .clang_arg("-v")
        .header("wrapper.h");

    // Let bindgen know about all include paths that were found.
    for path in include_paths {
        builder = builder.clang_arg(format!("-I{}", path));
    }

    let bindings = builder.generate().expect("Failed to generate bindings");

    bindings
        .write_to_file(out_path.as_ref().join("bindings.rs"))
        .expect("Failed to write bindings");
}

fn main() {
    let include_paths = run_pkg_config().expect("libevent not found");
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    generate_bindings(include_paths, out_path);
}
