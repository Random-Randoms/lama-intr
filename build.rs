use std::path::PathBuf;

fn make(path: &PathBuf) {
    if !std::process::Command::new("make")
        .arg("-C")
        .arg(path)
        .output()
        .expect("could not spawn `make`")
        .status
        .success()
    {
        panic!("could not compile object file");
    }
}

fn main() {
    let runtime_path = PathBuf::from("Lama/runtime")
        .canonicalize()
        .expect("cannot canonicalize path");
    let src = PathBuf::from("src")
        .canonicalize()
        .expect("cannot canonicalize path");
    let stub_c = src.join("stub.c");
    let stub_o = src.join("stub.o");
    let runtime_o = runtime_path.join("runtime.o");
    let gc_o = runtime_path.join("gc.o");
    let printf_o = runtime_path.join("printf.o");
    let lib_path = runtime_path.join("libruntime.a");

    println!("cargo:rustc-link-search={}", runtime_path.to_str().unwrap());
    println!("cargo:rustc-link-lib=runtime");
    println!("cargo:rustc-link-arg=-znostart-stop-gc");

    make(&runtime_path);

    if !std::process::Command::new("gcc")
        .arg("-c")
        .arg(stub_c)
        .arg("-o")
        .arg(stub_o.clone())
        .output()
        .unwrap()
        .status
        .success()
    {
        panic!("could't build stub.c");
    }

    if !std::process::Command::new("ar")
        .arg("rc")
        .arg(lib_path)
        .arg(runtime_o)
        .arg(gc_o)
        .arg(printf_o)
        .arg(stub_o)
        .output()
        .expect("could not spawn `ar`")
        .status
        .success()
    {
        panic!("could not emit library file");
    }
}
