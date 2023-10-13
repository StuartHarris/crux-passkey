use crux_core::typegen::TypeGen;
use shared::Counter;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=../shared");

    let mut gen = TypeGen::new();

    gen.register_app::<Counter>().expect("register");

    let output_root = PathBuf::from("./generated");

    gen.swift("SharedTypes", output_root.join("swift"))
        .expect("swift type gen failed");

    gen.java(
        "com.example.simple_counter.shared_types",
        output_root.join("java"),
    )
    .expect("java type gen failed");

    gen.typescript("shared_types", output_root.join("typescript"))
        .expect("typescript type gen failed");
}
