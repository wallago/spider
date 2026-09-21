//! Build script: compiles the Slint UI into Rust.

/// Turns `assets/ui.slint` into generated Rust under `OUT_DIR`.
#[allow(clippy::expect_used)]
fn main() {
    embuild::espidf::sysenv::output();

    slint_build::compile_with_config(
        "../assets/ui.slint",
        slint_build::CompilerConfiguration::new()
            .embed_resources(slint_build::EmbedResourcesKind::EmbedForSoftwareRenderer),
    )
    .expect("Slint UI failed to compile");
}
