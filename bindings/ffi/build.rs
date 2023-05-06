fn main() {
    uniffi_build::generate_scaffolding("./src/global_bindings.udl")
        .expect("Building the UDL file failed");
}
