//! main display module
use crate::command::Command;
use display_interface::{DataFormat::U8, DisplayError, WriteOnlyDataCommand};
use embedded_graphics::{
    pixelcolor::{Gray4, GrayColor},
    Pixel,
    prelude::{Size, OriginDimensions},
    draw_target::DrawTarget,
};
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;

const DISPLAY_WIDTH: usize = 128;
const DISPLAY_HEIGHT: usize = 128;
const BUFFER_SIZE: usize = DISPLAY_WIDTH * DISPLAY_HEIGHT * 4 / 8;

/// Represents the SSD1327 Display.
///
/// Use this struct to initialize the driver.
pub struct Ssd1327<DI> {
    display: DI,
    buffer: [u8; BUFFER_SIZE],
}

impl<DI: WriteOnlyDataCommand> Ssd1327<DI> {
    /// Creates the SSD1327 Display.
    ///
    /// Make sure to reset and initialize the display before use!
    pub fn new(display: DI) -> Self {
        Self {
            display,
            buffer: [0; BUFFER_SIZE],
        }
    }

    /// Resets the display.
    pub fn reset<RST, DELAY>(
        &mut self,
        rst: &mut RST,
        delay: &mut DELAY,
    ) -> Result<(), DisplayError>
    where
        RST: OutputPin,
        DELAY: DelayNs,
    {
        rst.set_high().map_err(|_| DisplayError::BusWriteError)?;
        delay.delay_ns(100_000_000);

        rst.set_low().map_err(|_| DisplayError::BusWriteError)?;
        delay.delay_ns(100_000_000);

        rst.set_high().map_err(|_| DisplayError::BusWriteError)?;
        delay.delay_ns(100_000_000);

        Ok(())
    }

    /// Initializes the display.
    pub fn init(&mut self) -> Result<(), DisplayError> {
        self.send_command(Command::DisplayOff)?;
        self.send_command(Command::ColumnAddress { start: 0, end: 127 })?;
        self.send_command(Command::RowAddress { start: 0, end: 127 })?;
        self.send_command(Command::Contrast(0x80))?;
        self.send_command(Command::SetRemap(0x51))?;
        self.send_command(Command::StartLine(0x00))?;
        self.send_command(Command::Offset(0x00))?;
        self.send_command(Command::DisplayModeNormal)?;
        self.send_command(Command::MuxRatio(0x7f))?;
        self.send_command(Command::PhaseLength(0xf1))?;
        self.send_command(Command::FrontClockDivider(0x00))?;
        self.send_command(Command::FunctionSelectionA(0x01))?;
        self.send_command(Command::SecondPreChargePeriod(0x0f))?;
        self.send_command(Command::ComVoltageLevel(0x0f))?;
        self.send_command(Command::PreChargeVoltage(0x08))?;
        self.send_command(Command::FunctionSelectionB(0x62))?;
        self.send_command(Command::CommandLock(0x12))?;
        self.send_command(Command::DisplayOn)?;

        Ok(())
    }

    /// Allows to send custom commands to the display.
    pub fn send_command(&mut self, command: Command) -> Result<(), DisplayError> {
        command.send(&mut self.display)
    }

    /// Flushes the display, and makes the output visible on the screen.
    pub fn flush(&mut self) -> Result<(), DisplayError> {
        self.display.send_data(U8(&self.buffer))
    }
}

impl<DI> DrawTarget for Ssd1327<DI> {
    type Error = DisplayError;
    type Color = Gray4;

    // fn draw_pixel(&mut self, pixel: Pixel<Gray4>) -> Result<(), Self::Error> {
    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
        where I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels.into_iter() {
            let index = ((coord.x / 2) + (coord.y * (DISPLAY_WIDTH / 2) as i32)) as usize;
            let old_val = self.buffer[index];
            let new_val: u8 = if coord.x % 2 == 0 {
                update_upper_half(old_val, color.luma())
            } else {
                update_lower_half(old_val, color.luma())
            };
            if new_val != old_val {
                self.buffer[index] = new_val;
            }
        }
        Ok(())
    }

    fn clear(&mut self, fill: Gray4) -> Result<(), Self::Error> {
        let luma = fill.luma();
        let byte = (luma << 4) | luma;
        self.buffer.fill(byte);
        Ok(())
    }
}
impl<DI> OriginDimensions for Ssd1327<DI> {
    fn size(&self) -> Size {
        Size::new(DISPLAY_WIDTH as u32, DISPLAY_HEIGHT as u32)
    }
}

#[inline]
fn update_upper_half(input: u8, color: u8) -> u8 {
    color << 4 | (input & 0x0F)
}

#[inline]
fn update_lower_half(input: u8, color: u8) -> u8 {
    color & 0x0f | (input & 0xF0)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn updates_upper_half_byte() {
        let input = 0b00000000;
        let color = 0b00001111;
        assert_eq!(0b11110000, update_upper_half(input, color));

        let input = 0b11110000;
        let color = 0b00000000;
        assert_eq!(0b00000000, update_upper_half(input, color));
    }

    #[test]
    fn leaves_lower_untouched_on_upper_change() {
        let input = 0b00000011;
        let color = 0b00001111;
        assert_eq!(0b11110011, update_upper_half(input, color));

        let input = 0b11111111;
        let color = 0b00000000;
        assert_eq!(0b00001111, update_upper_half(input, color));
    }

    #[test]
    fn updates_lower_half_byte() {
        let input = 0b00000000;
        let color = 0b00001111;
        assert_eq!(0b00001111, update_lower_half(input, color));

        let input = 0b00000000;
        let color = 0b00000000;
        assert_eq!(0b00000000, update_lower_half(input, color));
    }

    #[test]
    fn leaves_upper_untouched_on_lower_change() {
        let input = 0b11000011;
        let color = 0b00001111;
        assert_eq!(0b11001111, update_lower_half(input, color));

        let input = 0b11111111;
        let color = 0b00000000;
        assert_eq!(0b11110000, update_lower_half(input, color));
    }
}
