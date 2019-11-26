use cmake;

fn main() {
    let dst = cmake::Config::new("src/mainc").very_verbose(true).build();

    println!("cargo:rustc-link-search=native={}", dst.display());
    println!("cargo:rustc-link-lib=static=mainc");
}
