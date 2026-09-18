//! Build script: relays ESP-IDF's link settings from `esp-idf-sys` to this crate.

/// Emits the linker args and cfgs the ESP-IDF build produced.
fn main() {
    embuild::espidf::sysenv::output();
}
