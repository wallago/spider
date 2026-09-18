//! First light: bring up one 1.83" ST7789 panel and draw on it.

use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_10X20},
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::{Alignment, Text},
};
use esp_idf_svc::{
    hal::{
        delay::{Ets, FreeRtos},
        gpio::{AnyInputPin, PinDriver},
        peripherals::Peripherals,
        spi::{
            SpiDeviceDriver, SpiDriverConfig,
            config::{Config as SpiConfig, MODE_3},
        },
        units::FromValueType,
    },
    sys::EspError,
};
use log::{error, info};
use mipidsi::{Builder, interface::SpiInterface, models::ST7789, options::ColorInversion};

/// Visible width in pixels, portrait.
const WIDTH: u16 = 240;

/// Visible height in pixels.
const HEIGHT: u16 = 320;

/// Why first light failed.
#[derive(Debug)]
enum Error {
    /// ESP-IDF refused a driver: the SPI bus or a GPIO.
    Esp(EspError),
    /// The panel failed to init or draw. mipidsi's errors only implement `Debug`.
    Panel(String),
}

impl From<EspError> for Error {
    fn from(e: EspError) -> Self {
        Self::Esp(e)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Esp(e) => write!(f, "esp-idf: {e}"),
            Self::Panel(msg) => write!(f, "panel: {msg}"),
        }
    }
}

/// Boots, runs first light, logs the outcome.
fn main() {
    // Pulls in ESP-IDF runtime patches std relies on; must run before anything else.
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    if let Err(e) = run() {
        error!("first light failed: {e}");
    }
}

/// Brings the panel up, draws a test frame, then holds every driver alive.
///
/// Only returns on error: dropping a `PinDriver` resets its pin, which would
/// switch the backlight off the moment this function ended.
fn run() -> Result<(), Error> {
    let peripherals = Peripherals::take()?;
    let pins = peripherals.pins;

    // Backlight stays off until the panel holds a real frame — no garbage at boot.
    let mut backlight = PinDriver::output(pins.gpio6)?;
    backlight.set_low()?;
    let dc = PinDriver::output(pins.gpio22)?;
    let rst = PinDriver::output(pins.gpio23)?;

    // 10 MHz while bringing it up; raise once colours look clean.
    // No MISO: the panel never talks back.
    let spi = SpiDeviceDriver::new_single(
        peripherals.spi2,
        pins.gpio20, // CLK
        pins.gpio21, // DIN
        None::<AnyInputPin<'_>>,
        Some(pins.gpio5), // CS
        &SpiDriverConfig::new(),
        &SpiConfig::new().baudrate(10.MHz().into()).data_mode(MODE_3),
    )?;

    // Pixels are batched here before each SPI write.
    let mut buffer = [0_u8; 512];
    let di = SpiInterface::new(spi, dc, &mut buffer);

    let mut display = Builder::new(ST7789, di)
        // IPS ST7789 panels show a negative image without this.
        .invert_colors(ColorInversion::Inverted)
        .reset_pin(rst)
        .init(&mut Ets)
        .map_err(|e| Error::Panel(format!("init: {e:?}")))?;

    // Red first, over the whole RAM: every row the glass can show gets written,
    // so no power-on noise survives outside the visible window.
    // If it shows blue, the colour order is swapped (`.color_order(Bgr)`).
    display
        .clear(Rgb565::RED)
        .map_err(|e| Error::Panel(format!("clear: {e:?}")))?;
    backlight.set_high()?;
    info!("panel up, red frame sent");

    FreeRtos::delay_ms(1000);

    display
        .clear(Rgb565::BLACK)
        .map_err(|e| Error::Panel(format!("clear: {e:?}")))?;

    // The glass's window into RAM: origin at its first visible row, clipped to it.
    let mut visible = display.cropped(&Rectangle::new(
        Point::new(0, 0),
        Size::new(WIDTH.into(), HEIGHT.into()),
    ));

    Text::with_alignment(
        "spider",
        Point::new(i32::from(WIDTH) / 2, i32::from(HEIGHT) / 2),
        MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE),
        Alignment::Center,
    )
    .draw(&mut visible)
    .map_err(|e| Error::Panel(format!("text: {e:?}")))?;
    info!("test frame drawn");

    loop {
        FreeRtos::delay_ms(1000);
    }
}
