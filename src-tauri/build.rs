fn main() {
    println!("cargo:rerun-if-changed=resources/locales/");
    tauri_build::build()
}
