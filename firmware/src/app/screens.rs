use std::{
    cell::Cell,
    ops::Range,
    rc::Rc,
    time::{Duration, Instant},
};

use esp_idf_svc::{
    hal::{
        delay::{Ets, FreeRtos},
        gpio::{AnyInputPin, AnyOutputPin, Output, Pin, PinDriver},
        peripherals::Peripherals,
        spi::{
            SPI2, SpiDeviceDriver, SpiDriver, SpiDriverConfig,
            config::{Config as SpiConfig, MODE_3},
        },
        units::FromValueType,
    },
    sys::EspError,
};
use log::{error, info};
use mipidsi::{Builder, Display, interface::SpiInterface, models::ST7789, options::ColorInversion};
use slint::{
    ComponentHandle, PhysicalSize, PlatformError,
    platform::{
        Platform, WindowAdapter,
        software_renderer::{
            LineBufferProvider, MinimalSoftwareWindow, RepaintBufferType, Rgb565Pixel,
        },
    },
};

use crate::app::{Error, board::Board};

#[allow(
    missing_debug_implementations,
    missing_docs,
    unreachable_pub,
    unused_qualifications,
    clippy::pedantic
)]
mod ui {
    slint::include_modules!();
}

struct PinOut {
    clk: AnyOutputPin<'static>,
    din: AnyOutputPin<'static>,
    cs: AnyOutputPin<'static>,
    dc: AnyOutputPin<'static>,
    rst: AnyOutputPin<'static>,
    bl: AnyOutputPin<'static>,
    spi: SPI2<'static>,
}

pub(super) struct Screens {}

impl Screens {
    pub(super) fn new() -> Result<Self, Error> {
        let peripherals = Peripherals::take().map_err(|err| Error::Esp(err))?;
        Self::setup_devices(PinOut {
            bl: peripherals.pins.gpio32.into(),
            clk: peripherals.pins.gpio20.into(),
            cs: peripherals.pins.gpio27.into(),
            din: peripherals.pins.gpio21.into(),
            rst: peripherals.pins.gpio33.into(),
            dc: peripherals.pins.gpio46.into(),
            spi: peripherals.spi2.into(),
        })?;
        let peripherals = Peripherals::take().map_err(|err| Error::Esp(err))?;
        Self::setup_devices(PinOut {
            bl: peripherals.pins.gpio6.into(),
            clk: peripherals.pins.gpio20.into(),
            cs: peripherals.pins.gpio14.into(),
            din: peripherals.pins.gpio21.into(),
            rst: peripherals.pins.gpio5.into(),
            dc: peripherals.pins.gpio4.into(),
            spi: peripherals.spi2.into(),
        })?;

        let credit =
            ui::Panel1Display::new().map_err(|e| Error::Ui(format!("create panel 1: {e}")))?;
        credit
            .show()
            .map_err(|e| Error::Ui(format!("show panel 1: {e}")))?;

        let status =
            ui::Panel2Display::new().map_err(|e| Error::Ui(format!("create panel 2: {e}")))?;
        status
            .show()
            .map_err(|e| Error::Ui(format!("show panel 2: {e}")))?;

        Ok(Self {})
    }

    fn set_backlight(bl: AnyOutputPin<'static>) -> Result<(), Error> {
        let mut backlight = PinDriver::output(bl).map_err(|err| Error::Esp(err))?;
        backlight.set_low().map_err(|err| Error::Esp(err))?;
        Ok(())
    }

    fn set_common_bus(
        spi: SPI2<'static>,
        clk: AnyOutputPin<'static>,
        din: AnyOutputPin<'static>,
    ) -> Result<SpiDriver<'static>, Error> {
        let bus = SpiDriver::new(
            spi,
            clk,
            din,
            None::<AnyInputPin<'_>>,
            &SpiDriverConfig::new(),
        )
        .map_err(|err| Error::Esp(err))?;
        Ok(bus)
    }

    fn set_device(
        bus: &'static SpiDriver<'static>,
        cs: AnyOutputPin<'static>,
    ) -> Result<SpiDeviceDriver<'static, &'static SpiDriver<'static>>, Error> {
        let spi_config = SpiConfig::new().baudrate(10.MHz().into()).data_mode(MODE_3);
        let device =
            SpiDeviceDriver::new(bus, Some(cs), &spi_config).map_err(|err| Error::Esp(err))?;
        Ok(device)
    }

    fn set_display(
        dc: AnyOutputPin<'static>,
        rst: AnyOutputPin<'static>,
        device: SpiDeviceDriver<'static, &'static SpiDriver<'static>>,
        buffer: &'static mut [u8; 512],
    ) -> Result<
        Display<
            SpiInterface<
                'static,
                SpiDeviceDriver<'static, &'static SpiDriver<'static>>,
                PinDriver<'static, Output>,
            >,
            ST7789,
            PinDriver<'static, Output>,
        >,
        Error,
    > {
        let display = Builder::new(
            ST7789,
            SpiInterface::new(device, PinDriver::output(dc)
            .map_err(|err| Error::Esp(err))?
                , buffer),
        )
        // IPS ST7789 panels show a negative image without this.
        .invert_colors(ColorInversion::Inverted)
        .reset_pin(PinDriver::output(rst)
            .map_err(|err| Error::Esp(err))?
            )
        .init(&mut Ets)
        .map_err(|e| Error::Panel(format!("init panel 1: {e:?}")))?;
        Ok(display)
    }

    fn setup_devices(pins: PinOut) -> Result<(), Error> {
        Self::set_backlight(pins.bl)?;
        let bus: &'static SpiDriver<'static> = Box::leak(Box::new(Self::set_common_bus(
            pins.spi, pins.clk, pins.din,
        )?));
        let device = Self::set_device(&bus, pins.cs)?;
        let buffer: &'static mut [u8; 512] = Box::leak(Box::new([0_u8; 512]));
        let display = Self::set_display(pins.dc, pins.rst, device, buffer)?;
        Ok(())
    }

    fn set_windows() -> Result<(), Error> {
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
        Ok(())
    }
}
