use std::{cell::Cell, rc::Rc, time::Instant};

use esp_idf_svc::{
    hal::{delay::FreeRtos, gpio::AnyOutputPin, peripherals::Peripherals, spi::SPI2},
    sys::EspError,
};
use log::info;
use slint::{
    ComponentHandle, PhysicalSize,
    platform::software_renderer::{MinimalSoftwareWindow, RepaintBufferType},
};

use crate::app::{board::Board, screen::Screen};

mod board;
mod draw;
mod screen;
mod ui {
    slint::include_modules!();
}

#[derive(Debug)]
pub enum Error {
    /// ESP-IDF refused a driver: the SPI bus or a GPIO.
    Esp(EspError),
    /// The panel failed to init or draw. mipidsi's errors only implement `Debug`.
    Panel(String),
    /// Slint refused the platform, or the UI failed to build.
    Ui(String),
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
            Self::Ui(msg) => write!(f, "ui: {msg}"),
        }
    }
}

/// The SPI bus both panels share.
struct BusPins {
    spi: SPI2<'static>,
    clk: AnyOutputPin<'static>,
    din: AnyOutputPin<'static>,
}

/// The pins each panel owns on its own.
struct PanelPins {
    cs: AnyOutputPin<'static>,
    dc: AnyOutputPin<'static>,
    rst: AnyOutputPin<'static>,
    bl: AnyOutputPin<'static>,
}

/// Runs the widget. Never returns: there is no caller to return to.
pub(crate) fn run() -> Result<(), Error> {
    let peripherals = Peripherals::take()?;
    let bus = Screen::bus(BusPins {
        spi: peripherals.spi2,
        clk: peripherals.pins.gpio21.into(),
        din: peripherals.pins.gpio20.into(),
    })?;
    let mut screen_1 = Screen::new(
        bus,
        PanelPins {
            cs: peripherals.pins.gpio27.into(),
            dc: peripherals.pins.gpio46.into(),
            rst: peripherals.pins.gpio33.into(),
            bl: peripherals.pins.gpio32.into(),
        },
    )?;
    let mut screen_2 = Screen::new(
        bus,
        PanelPins {
            cs: peripherals.pins.gpio14.into(),
            dc: peripherals.pins.gpio4.into(),
            rst: peripherals.pins.gpio5.into(),
            bl: peripherals.pins.gpio6.into(),
        },
    )?;

    // Slint needs its platform before the first component is created;
    // each `new()` below takes the next window, in order.
    let windows = set_windows()?;

    let ui_1 = ui::Panel1Display::new().map_err(|e| Error::Ui(format!("create panel 1: {e}")))?;
    ui_1.show()
        .map_err(|e| Error::Ui(format!("show panel 1: {e}")))?;

    let ui_2 = ui::Panel2Display::new().map_err(|e| Error::Ui(format!("create panel 2: {e}")))?;
    ui_2.show()
        .map_err(|e| Error::Ui(format!("show panel 2: {e}")))?;

    let mut lit = false;

    loop {
        slint::platform::update_timers_and_animations();

        if let Some(window) = windows.get(0) {
            draw::draw(window, &mut screen_1.display)?;
        }
        if let Some(window) = windows.get(1) {
            draw::draw(window, &mut screen_2.display)?;
        }

        if !lit {
            screen_1.backlight.set_high()?;
            screen_2.backlight.set_high()?;
            lit = true;
            info!("first frame drawn on 2 panels");
        }

        // Animations need frames; otherwise idle at ~60 Hz polling.
        if !windows.iter().any(|window| window.has_active_animations()) {
            FreeRtos::delay_ms(16);
        }
    }
}

fn set_windows() -> Result<[Rc<MinimalSoftwareWindow>; 2], Error> {
    let windows: [Rc<MinimalSoftwareWindow>; 2] =
        std::array::from_fn(|_| MinimalSoftwareWindow::new(RepaintBufferType::ReusedBuffer));
    for window in &windows {
        window.set_size(PhysicalSize::new(240, 284));
    }

    slint::platform::set_platform(Box::new(Board {
        windows: windows.clone(),
        handed_out: Cell::new(0),
        start: Instant::now(),
    }))
    .map_err(|e| Error::Ui(format!("platform: {e:?}")))?;
    Ok(windows)
}
