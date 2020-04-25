use cc;

fn main() {
    println!("cargo:rerun-if-changed=src/evhack/evhack.c");
    cc::Build::new()
        .file("src/evhack/evhack.c")
        .compile("libevhack.a");
}