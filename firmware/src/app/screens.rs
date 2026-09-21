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

pub(super) struct Screens {
    peripherals: Peripherals,
}

impl Screens {
    pub(super) fn new() -> Result<Self, Error> {
        Ok(Self {
            peripherals: Peripherals::take().map_err(|err| Error::ESP(err.code()))?,
        })
    }

    pub(super) fn init(self) -> Result<(), Error> {
        self.set_backlight()
    }

    fn set_backlight(self) -> Result<(), Error> {
        let mut backlights = [
            PinDriver::output(self.peripherals.pins.gpio1).map_err(|err| Error::ESP(err.code()))?,
            PinDriver::output(self.peripherals.pins.gpio33)
                .map_err(|err| Error::ESP(err.code()))?,
        ];
        for backlight in &mut backlights {
            backlight.set_low().map_err(|err| Error::ESP(err.code()))?;
        }
        Ok(())
    }
}
