fn main() {
    let slint_entry = "ui/app.slint";
    let config = slint_build::CompilerConfiguration::new().with_style("fluent".into());

    if let Err(e) = slint_build::compile_with_config(slint_entry, config) {
        panic!("Failed to compile Slint UI file {}: {:?}", slint_entry, e);
    }
}
