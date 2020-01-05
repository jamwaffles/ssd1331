use crate::command::{AddressIncrementMode, ColorMode, Command, VcomhLevel};
use crate::displayrotation::DisplayRotation;
use crate::{DISPLAY_HEIGHT, DISPLAY_WIDTH};
use embedded_hal::digital::v2::OutputPin;

/// Container to store and set display properties
pub struct Properties<SPI, DC> {
    spi: SPI,
    dc: DC,
    display_rotation: DisplayRotation,
}

impl<SPI, DC> Properties<SPI, DC>
where
    SPI: hal::blocking::spi::Write<u8>,
    DC: OutputPin,
{
    /// Create new Properties instance
    pub fn new(spi: SPI, dc: DC, display_rotation: DisplayRotation) -> Self {
        Properties {
            spi,
            dc,
            display_rotation,
        }
    }

    /// Initialise the display in column mode (i.e. a byte walks down a column of 8 pixels) with
    /// column 0 on the left and column _(display_width - 1)_ on the right.
    pub fn init_column_mode(&mut self) -> Result<(), ()> {
        let display_rotation = self.display_rotation;

        Command::DisplayOn(false).send(&mut self.spi, &mut self.dc)?;
        Command::DisplayClockDiv(0x8, 0x0).send(&mut self.spi, &mut self.dc)?;
        Command::Multiplex(64 - 1).send(&mut self.spi, &mut self.dc)?;
        Command::DisplayOffset(0).send(&mut self.spi, &mut self.dc)?;
        Command::StartLine(0).send(&mut self.spi, &mut self.dc)?;

        self.set_rotation(display_rotation)?;

        // Values taken from [here](https://github.com/adafruit/Adafruit-SSD1331-OLED-Driver-Library-for-Arduino/blob/master/Adafruit_SSD1331.cpp#L119-L124)
        Command::Contrast(0x91, 0x50, 0x7D).send(&mut self.spi, &mut self.dc)?;
        Command::PreChargePeriod(0x1, 0xF).send(&mut self.spi, &mut self.dc)?;
        Command::VcomhDeselect(VcomhLevel::V071).send(&mut self.spi, &mut self.dc)?;
        Command::AllOn(false).send(&mut self.spi, &mut self.dc)?;
        Command::Invert(false).send(&mut self.spi, &mut self.dc)?;
        Command::DisplayOn(true).send(&mut self.spi, &mut self.dc)?;

        Ok(())
    }

    /// Set the position in the framebuffer of the display where any sent data should be
    /// drawn. This method can be used for changing the affected area on the screen as well
    /// as (re-)setting the start point of the next `draw` call.
    pub fn set_draw_area(&mut self, start: (u8, u8), end: (u8, u8)) -> Result<(), ()> {
        Command::ColumnAddress(start.0, end.0 - 1).send(&mut self.spi, &mut self.dc)?;
        Command::RowAddress(start.1.into(), (end.1 - 1).into())
            .send(&mut self.spi, &mut self.dc)?;
        Ok(())
    }

    /// Send the data to the display for drawing at the current position in the framebuffer
    /// and advance the position accordingly. Cf. `set_draw_area` to modify the affected area by
    /// this method.
    pub fn draw(&mut self, buffer: &[u8]) -> Result<(), ()> {
        // 1 = data, 0 = command
        self.dc.set_high().map_err(|_| ())?;

        self.spi.write(&buffer).map_err(|_| ())?;

        Ok(())
    }

    /// Get display dimensions, taking into account the current rotation of the display
    ///
    /// # Examples
    ///
    /// ## No rotation
    ///
    /// ```rust
    /// # use ssd1331::test_helpers::{Spi, Pin, Properties};
    /// use ssd1331::{DisplayRotation, Builder};
    ///
    /// // Set up SPI interface and digital pin. These are stub implementations used in examples.
    /// let spi = Spi;
    /// let dc = Pin;
    ///
    /// let properties = Properties::new(
    ///     spi,
    ///     dc,
    ///     DisplayRotation::Rotate0
    /// );
    ///
    /// assert_eq!(properties.dimensions(), (96, 64));
    /// ```
    ///
    /// ## 90 degree rotation rotation
    ///
    /// ```rust
    /// # use ssd1331::test_helpers::{Spi, Pin, Properties};
    /// use ssd1331::{DisplayRotation, Builder};
    ///
    /// // Set up SPI interface and digital pin. These are stub implementations used in examples.
    /// let spi = Spi;
    /// let dc = Pin;
    ///
    /// let properties = Properties::new(
    ///     spi,
    ///     dc,
    ///     DisplayRotation::Rotate90
    /// );
    ///
    /// assert_eq!(properties.dimensions(), (64, 96));
    /// ```
    pub fn dimensions(&self) -> (u8, u8) {
        match self.display_rotation {
            DisplayRotation::Rotate0 | DisplayRotation::Rotate180 => {
                (DISPLAY_WIDTH, DISPLAY_HEIGHT)
            }
            DisplayRotation::Rotate90 | DisplayRotation::Rotate270 => {
                (DISPLAY_HEIGHT, DISPLAY_WIDTH)
            }
        }
    }

    /// Get the display rotation
    pub fn rotation(&self) -> DisplayRotation {
        self.display_rotation
    }

    /// Set the display rotation
    pub fn set_rotation(&mut self, display_rotation: DisplayRotation) -> Result<(), ()> {
        self.display_rotation = display_rotation;

        match display_rotation {
            DisplayRotation::Rotate0 => {
                Command::RemapAndColorDepth(
                    false,
                    false,
                    ColorMode::CM65k,
                    AddressIncrementMode::Horizontal,
                )
                .send(&mut self.spi, &mut self.dc)?;
            }
            DisplayRotation::Rotate90 => {
                Command::RemapAndColorDepth(
                    true,
                    false,
                    ColorMode::CM65k,
                    AddressIncrementMode::Vertical,
                )
                .send(&mut self.spi, &mut self.dc)?;
            }
            DisplayRotation::Rotate180 => {
                Command::RemapAndColorDepth(
                    true,
                    true,
                    ColorMode::CM65k,
                    AddressIncrementMode::Horizontal,
                )
                .send(&mut self.spi, &mut self.dc)?;
            }
            DisplayRotation::Rotate270 => {
                Command::RemapAndColorDepth(
                    false,
                    true,
                    ColorMode::CM65k,
                    AddressIncrementMode::Vertical,
                )
                .send(&mut self.spi, &mut self.dc)?;
            }
        };

        Ok(())
    }
}
