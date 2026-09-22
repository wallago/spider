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

use crate::app::Error;

/// One ST7789 on the shared bus, with its own D/C and reset pins.
///
/// Both panels land on this exact type: `PinDriver` erases the pin it drives,
/// and both SPI devices borrow the same `SpiDriver`.
type Panel<'d> = mipidsi::Display<
    SpiInterface<'d, SpiDeviceDriver<'d, &'d SpiDriver<'d>>, PinDriver<'d, Output>>,
    ST7789,
    PinDriver<'d, Output>,
>;

/// Lends Slint one panel row at a time, then hands the rendered part to `push`.
struct LineSink<F> {
    /// One panel row; Slint renders into the dirty part of it.
    buffer: Vec<Rgb565Pixel>,
    /// Receives the row index, the dirty column range, and its pixels.
    push: F,
}

impl<F: FnMut(usize, Range<usize>, &[Rgb565Pixel])> LineBufferProvider for LineSink<F> {
    type TargetPixel = Rgb565Pixel;

    fn process_line(
        &mut self,
        line: usize,
        range: Range<usize>,
        render_fn: impl FnOnce(&mut [Self::TargetPixel]),
    ) {
        let pixels = &mut self.buffer[range.clone()];
        render_fn(pixels);
        (self.push)(line, range, pixels);
    }
}

/// Renders `window` into `display`, if Slint says it needs a repaint.
pub(super) fn draw(window: &MinimalSoftwareWindow, display: &mut Panel<'_>) -> Result<(), Error> {
    // `process_line` can't return an error, so the first one is parked here.
    let mut failed = None;
    window.draw_if_needed(|renderer| {
        renderer.render_by_line(LineSink {
            buffer: vec![Rgb565Pixel::default(); 240],
            push: |line, range: Range<usize>, pixels: &[Rgb565Pixel]| {
                if range.is_empty() {
                    return;
                }
                let (Ok(x0), Ok(x1), Ok(y)) = (
                    u16::try_from(range.start),
                    u16::try_from(range.end - 1),
                    u16::try_from(line),
                ) else {
                    return;
                };
                // Same 565 bit layout on both sides; mipidsi does the byte order.
                let colors = pixels.iter().map(|p| Rgb565::from(RawU16::new(p.0)));
                // `set_pixels` takes inclusive end coordinates.
                if let Err(e) = display.set_pixels(x0, y, x1, y, colors) {
                    failed.get_or_insert(format!("line {line}: {e:?}"));
                }
            },
        });
    });
    if let Some(msg) = failed {
        return Err(Error::Panel(msg));
    }
    Ok(())
}
