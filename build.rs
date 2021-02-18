extern crate cc;

fn main() {
    cc::Build::new()
        .file("src_c/hand_index.c")
        .file("src_c/rust_interface.c")
        .include("src_c")
        .compile("hand_index");
}
