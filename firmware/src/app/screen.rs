use esp_idf_svc::hal::{
    delay::Ets,
    gpio::{AnyInputPin, AnyOutputPin, Output, PinDriver},
    spi::{
        SPI2, SpiDeviceDriver, SpiDriver, SpiDriverConfig,
        config::{Config as SpiConfig, MODE_3},
    },
    units::FromValueType,
};
use mipidsi::{Builder, Display, interface::SpiInterface, models::ST7789, options::ColorInversion};

use crate::app::{BusPins, Error, PanelPins};

pub(super) struct Screen {
    pub(super) backlight: PinDriver<'static, Output>,
    pub(super) display: Display<
        SpiInterface<
            'static,
            SpiDeviceDriver<'static, &'static SpiDriver<'static>>,
            PinDriver<'static, Output>,
        >,
        ST7789,
        PinDriver<'static, Output>,
    >,
}

impl Screen {
    /// Installs the shared SPI bus once; every panel borrows it for good.
    pub(super) fn bus(pins: BusPins) -> Result<&'static SpiDriver<'static>, Error> {
        let bus = Self::set_common_bus(pins.spi, pins.clk, pins.din)?;
        Ok(Box::leak(Box::new(bus)))
    }

    pub(super) fn new(bus: &'static SpiDriver<'static>, pins: PanelPins) -> Result<Self, Error> {
        let backlight = Self::set_backlight(pins.bl)?;
        let device = Self::set_device(bus, pins.cs)?;
        let buffer: &'static mut [u8; 512] = Box::leak(Box::new([0_u8; 512]));
        let display = Self::set_display(pins.dc, pins.rst, device, buffer)?;

        Ok(Self { backlight, display })
    }

    fn set_backlight(bl: AnyOutputPin<'static>) -> Result<PinDriver<'static, Output>, Error> {
        let mut backlight = PinDriver::output(bl).map_err(|err| Error::Esp(err))?;
        backlight.set_low().map_err(|err| Error::Esp(err))?;
        Ok(backlight)
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
}
