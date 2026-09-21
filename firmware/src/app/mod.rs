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
use spider_core::error::Error;

use crate::app::screens::Screens;

mod screens;

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
    Ok(())
}
