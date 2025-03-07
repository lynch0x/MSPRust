use cc::Build;

fn main() {
    // Compile the C++ code into a static library (libmy_c_code.a)
    Build::new()
    .cpp(true).file("src/cpp/dllmain.cpp")  
      .compile("mspclient");  
    // Link the `wininet` library directly in build.rs (for MinGW)
    println!("cargo:rustc-link-lib=dylib=wininet");
}
