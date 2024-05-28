fn main() {
    println!("cargo:rerun-if-changed=../shared/conf.h");
    let bindings = bindgen::Builder::default()
        .header("../shared/conf.h")
        .clang_arg("-I../shared") // Specify the include path for additional headers if needed
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file("src/config/raw_bindings.rs")
        .expect("Couldn't write bindings!");
}

