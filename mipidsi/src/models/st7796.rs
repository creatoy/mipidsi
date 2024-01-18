use display_interface::{DataFormat, WriteOnlyDataCommand};
use embedded_graphics_core::{pixelcolor::Rgb565, prelude::IntoStorage};
use embedded_hal::{delay::DelayNs, digital::OutputPin};

use crate::{
    dcs::{
        BitsPerPixel, Dcs, EnterNormalMode, ExitSleepMode, PixelFormat, SetAddressMode,
        SetDisplayOn, SetInvertMode, SetPixelFormat, SetScrollArea, SoftReset, WriteMemoryStart,
    },
    error::InitError,
    Builder, Error, ModelOptions,
};

use super::Model;

/// ST7796 display in Rgb565 color mode.
///
/// Interfaces implemented by the [display-interface](https://crates.io/crates/display-interface) are supported.
pub struct ST7796;

impl Model for ST7796 {
    type ColorFormat = Rgb565;

    fn init<RST, DELAY, DI>(
        &mut self,
        dcs: &mut Dcs<DI>,
        delay: &mut DELAY,
        options: &ModelOptions,
        rst: &mut Option<RST>,
    ) -> Result<SetAddressMode, InitError<RST::Error>>
    where
        RST: OutputPin,
        DELAY: DelayNs,
        DI: WriteOnlyDataCommand,
    {
        match rst {
            Some(ref mut rst) => self.hard_reset(rst, delay)?,
            None => dcs.write_command(SoftReset)?,
        }
        delay.delay_us(150_000);

        dcs.write_command(ExitSleepMode)?;
        delay.delay_us(10_000);

        dcs.write_raw(0xF0, &[0xC3])?;
        dcs.write_raw(0xF0, &[0x96])?;

        // set hw scroll area based on framebuffer size
        dcs.write_command(SetScrollArea::from(options))?;
        let madctl = SetAddressMode::from(options);
        dcs.write_command(madctl)?;

        dcs.write_command(SetInvertMode(options.invert_colors))?;

        let pf = PixelFormat::with_all(BitsPerPixel::from_rgb_color::<Self::ColorFormat>());
        dcs.write_command(SetPixelFormat::new(pf))?;
        delay.delay_us(10_000);

        dcs.write_raw(0xB4, &[0x01])?; // 1-dot inversion
        dcs.write_raw(0xB7, &[0xC6])?;

        dcs.write_raw(0xC1, &[0x15])?;
        dcs.write_raw(0xC2, &[0xAF])?;
        dcs.write_raw(0xC5, &[0x22])?;
        dcs.write_raw(0xC6, &[0x00])?;
        dcs.write_raw(0xE8, &[0x40, 0x8A, 0x00, 0x00, 0x29, 0x19, 0xA5, 0x33])?;

        // Gamma correction
        dcs.write_raw(
            0xE0,
            &[
                0xF0, 0x04, 0x08, 0x09, 0x08, 0x15, 0x2F, 0x42, 0x46, 0x28, 0x15, 0x16, 0x29, 0x2D,
            ],
        )?;
        dcs.write_raw(
            0xE0,
            &[
                0xF0, 0x04, 0x09, 0x09, 0x08, 0x15, 0x2E, 0x46, 0x46, 0x28, 0x15, 0x15, 0x29, 0x2D,
            ],
        )?;

        dcs.write_command(EnterNormalMode)?;
        delay.delay_us(10_000);
        dcs.write_raw(0x53, &[0x24])?;
        dcs.write_raw(0xF0, &[0x3C])?;
        dcs.write_raw(0xF0, &[0x69])?;
        dcs.write_command(SetDisplayOn)?;

        // DISPON requires some time otherwise we risk SPI data issues
        delay.delay_us(120_000);

        Ok(madctl)
    }

    fn write_pixels<DI, I>(&mut self, dcs: &mut Dcs<DI>, colors: I) -> Result<(), Error>
    where
        DI: WriteOnlyDataCommand,
        I: IntoIterator<Item = Self::ColorFormat>,
    {
        dcs.write_command(WriteMemoryStart)?;

        let mut iter = colors.into_iter().map(Rgb565::into_storage);

        let buf = DataFormat::U16BEIter(&mut iter);
        dcs.di.send_data(buf)?;
        Ok(())
    }

    fn default_options() -> crate::ModelOptions {
        ModelOptions::with_sizes((320, 480), (320, 480))
    }
}

// simplified constructor on Display

impl<DI> Builder<DI, ST7796>
where
    DI: WriteOnlyDataCommand,
{
    /// Creates a new display builder for a ST7796 display in Rgb565 color mode.
    ///
    /// The default framebuffer size and display size is 240x320 pixels.
    ///
    /// # Arguments
    ///
    /// * `di` - a [display interface](WriteOnlyDataCommand) for communicating with the display
    ///
    pub fn st7796(di: DI) -> Self {
        Self::with_model(di, ST7796)
    }
}
