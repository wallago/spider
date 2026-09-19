//! Build script: relays ESP-IDF's link settings from `esp-idf-sys` to this crate,
//! and compiles the Slint UI into Rust.

/// Emits the linker args and cfgs the ESP-IDF build produced, then the UI.
fn main() {
    embuild::espidf::sysenv::output();

    // Fonts are rasterised here, at the sizes the UI uses, as anti-aliased
    // glyphs — no font engine runs on the chip.
    slint_build::compile_with_config(
        "../assets/credit_dashboard.slint",
        slint_build::CompilerConfiguration::new()
            .embed_resources(slint_build::EmbedResourcesKind::EmbedForSoftwareRenderer),
    )
    .expect("Slint UI failed to compile");
}
