use std::{
    cell::Cell,
    ops::Range,
    rc::Rc,
    time::{Duration, Instant},
};

use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::pixelcolor::{Rgb565, raw::RawU16};
use embedded_graphics::prelude::RgbColor;
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

/// Slint's view of the board: one fixed window per panel, and a clock.
pub(super) struct Board {
    /// One per panel. Slint takes these in order, one per UI that gets created.
    pub(super) windows: [Rc<MinimalSoftwareWindow>; 2],
    /// How many of `windows` Slint has already taken.
    pub(super) handed_out: Cell<usize>,
    /// Boot time, the zero for Slint's timers and animations.
    pub(super) start: Instant,
}

impl Platform for Board {
    fn create_window_adapter(&self) -> Result<Rc<dyn WindowAdapter>, PlatformError> {
        let taken = self.handed_out.get();
        let window = self.windows.get(taken).ok_or_else(|| {
            PlatformError::Other(format!("only 2 panels, but UI {taken} asked for a window"))
        })?;
        self.handed_out.set(taken + 1);
        Ok(window.clone())
    }

    fn duration_since_start(&self) -> Duration {
        self.start.elapsed()
    }
}
