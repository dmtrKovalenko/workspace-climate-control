use std::path::Path;

fn main() {
    // if it is not keep using the last one
    if Path::new("../shared/conf.h").exists() {
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
}
