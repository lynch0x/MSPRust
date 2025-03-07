fn main() {
    println!("cargo:rustc-link-search=native=lib"); // Path where .lib is located
    println!("cargo:rustc-link-lib=static=MSPClient");  // Link the .lib file (without extension)
}