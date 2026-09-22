//! Bring up two 1.83" ST7789 panels on one SPI bus and drive a Slint UI on each.

use log::error;

/// Main application.
mod app;

/// Boots, runs the UI, logs why it stopped.
fn main() {
    // Pulls in ESP-IDF runtime patches std relies on; must run before anything else.
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    if let Err(e) = app::run() {
        error!("Firmware stopped: {e}");
    }
}
