use std::{
    cell::Cell,
    ops::Range,
    rc::Rc,
    time::{Duration, Instant},
};

use esp_idf_svc::{
    hal::{
        delay::{Ets, FreeRtos},
        gpio::{AnyInputPin, Output, PinDriver},
        peripherals::Peripherals,
        spi::{
            SpiDeviceDriver, SpiDriver, SpiDriverConfig,
            config::{Config as SpiConfig, MODE_3},
        },
        units::FromValueType,
    },
    sys::EspError,
};
use log::{error, info};
use mipidsi::{Builder, interface::SpiInterface, models::ST7789, options::ColorInversion};
use slint::{
    ComponentHandle, PhysicalSize, PlatformError,
    platform::{
        Platform, WindowAdapter,
        software_renderer::{
            LineBufferProvider, MinimalSoftwareWindow, RepaintBufferType, Rgb565Pixel,
        },
    },
};

use crate::app::screens::Screens;

mod board;
mod screens;

#[derive(Debug)]
enum Error {
    /// ESP-IDF refused a driver: the SPI bus or a GPIO.
    Esp(EspError),
    /// The panel failed to init or draw. mipidsi's errors only implement `Debug`.
    Panel(String),
    /// Slint refused the platform, or the UI failed to build.
    Ui(String),
}

/// Runs the widget. Never returns: there is no caller to return to.
pub(crate) fn run(peripherals: Peripherals) -> Result<(), Error> {
    let screens = Screens::new()?;
    // let pins = peripherals.pins;
    // let delay = Delay::new();
    //
    // if let Err(e) = widget(peripherals, delay) {
    //     // Nothing above this can act on the error. Report it and stop, leaving
    //     // whatever is on the glass where it is.
    //     error!("canary stopped: {e}");
    // }
    //
    // loop {
    //     delay.delay_millis(1_000);
    // }

    let mut lit = false;

    loop {
        slint::platform::update_timers_and_animations();

        for (window, display) in windows.iter().zip(displays.iter_mut()) {
            draw(window, display)?;
        }

        if !lit {
            // The pass above drew every panel in full, so none shows garbage.
            for backlight in &mut backlights {
                backlight.set_high()?;
            }
            lit = true;
            info!("first frame drawn on {PANELS} panels");
        }

        // Animations need frames; otherwise idle at ~60 Hz polling.
        if !windows.iter().any(|window| window.has_active_animations()) {
            FreeRtos::delay_ms(16);
        }
    }

    Ok(())
}
