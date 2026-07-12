//! Waveshare 3inch e-Paper (G) support.
//!
//! The command sequence and 2-bit color packing contract are ported from
//! Waveshare's MIT-licensed `EPD_3in0g` C/Python demos.

use anyhow::{bail, Result};
use image::Luma;

#[cfg(target_os = "linux")]
use std::time::{Duration, Instant};

#[cfg(target_os = "linux")]
use anyhow::{anyhow, Context};

use crate::render::MonoFrame;

pub const PANEL_WIDTH: u16 = 168;
pub const PANEL_HEIGHT: u16 = 400;
pub const LANDSCAPE_WIDTH: u16 = 400;
pub const LANDSCAPE_HEIGHT: u16 = 168;
pub const PACKED_BUFFER_LEN: usize = (PANEL_WIDTH as usize * PANEL_HEIGHT as usize) / 4;
#[cfg(target_os = "linux")]
const SPI_CHUNK_SIZE: usize = 4096;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Epd3in0gColor {
    Black = 0,
    White = 1,
    Yellow = 2,
    Red = 3,
}

impl Epd3in0gColor {
    pub fn repeated_byte(self) -> u8 {
        let value = self as u8;
        (value << 6) | (value << 4) | (value << 2) | value
    }
}

pub fn pack_landscape_mono_frame(frame: &MonoFrame) -> Result<Vec<u8>> {
    if frame.width() != LANDSCAPE_WIDTH as u32 || frame.height() != LANDSCAPE_HEIGHT as u32 {
        bail!(
            "Waveshare 3inch G frame must be {}x{}, got {}x{}",
            LANDSCAPE_WIDTH,
            LANDSCAPE_HEIGHT,
            frame.width(),
            frame.height()
        );
    }

    let mut unpacked =
        vec![Epd3in0gColor::White as u8; PANEL_WIDTH as usize * PANEL_HEIGHT as usize];

    for source_y in 0..LANDSCAPE_HEIGHT as u32 {
        for source_x in 0..LANDSCAPE_WIDTH as u32 {
            // Match Waveshare's Python sample: a 400x168 landscape image is rotate(90, expand=True)
            // before packing into the controller's native 168x400 portrait buffer.
            let target_x = source_y;
            let target_y = LANDSCAPE_WIDTH as u32 - 1 - source_x;
            let idx = (target_y as usize * PANEL_WIDTH as usize) + target_x as usize;
            unpacked[idx] = mono_pixel_to_epd_color(*frame.get_pixel(source_x, source_y)) as u8;
        }
    }

    Ok(pack_2bpp_pixels(&unpacked))
}

pub fn pack_2bpp_pixels(pixels: &[u8]) -> Vec<u8> {
    pixels
        .chunks(4)
        .map(|chunk| {
            let a = chunk.first().copied().unwrap_or(Epd3in0gColor::White as u8) & 0x03;
            let b = chunk.get(1).copied().unwrap_or(Epd3in0gColor::White as u8) & 0x03;
            let c = chunk.get(2).copied().unwrap_or(Epd3in0gColor::White as u8) & 0x03;
            let d = chunk.get(3).copied().unwrap_or(Epd3in0gColor::White as u8) & 0x03;
            (a << 6) | (b << 4) | (c << 2) | d
        })
        .collect()
}

fn mono_pixel_to_epd_color(pixel: Luma<u8>) -> Epd3in0gColor {
    if pixel[0] < 128 {
        Epd3in0gColor::Black
    } else {
        Epd3in0gColor::White
    }
}

#[cfg(target_os = "linux")]
#[derive(Debug)]
pub struct Epd3in0gDevice {
    spi: rppal::spi::Spi,
    reset: rppal::gpio::OutputPin,
    dc: rppal::gpio::OutputPin,
    busy: rppal::gpio::InputPin,
    sleeping: bool,
}

#[cfg(target_os = "linux")]
impl Epd3in0gDevice {
    pub fn open_default() -> Result<Self> {
        use rppal::gpio::Gpio;
        use rppal::spi::{Bus, Mode, SlaveSelect, Spi};

        let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 4_000_000, Mode::Mode0)
            .context("opening SPI0 CE0 for Waveshare 3inch G")?;
        let gpio = Gpio::new().context("opening Raspberry Pi GPIO")?;
        let reset = gpio
            .get(17)
            .context("opening Waveshare reset pin GPIO17")?
            .into_output();
        let dc = gpio
            .get(25)
            .context("opening Waveshare data/command pin GPIO25")?
            .into_output();
        let busy = gpio
            .get(24)
            .context("opening Waveshare busy pin GPIO24")?
            .into_input();

        Ok(Self {
            spi,
            reset,
            dc,
            busy,
            sleeping: false,
        })
    }

    pub fn init(&mut self) -> Result<()> {
        self.sleeping = false;
        self.reset();
        self.command_with_data(0x66, &[0x49, 0x55, 0x13, 0x5D, 0x05, 0x10])?;
        self.command_with_data(0xB0, &[0x00])?;
        self.command_with_data(0x01, &[0x0F, 0x00])?;
        self.command_with_data(0x00, &[0x4F, 0x6B])?;
        self.command_with_data(0x06, &[0xD7, 0xDE, 0x12])?;
        self.command_with_data(0x61, &[0x00, 0xA8, 0x01, 0x90])?;
        self.command_with_data(0x50, &[0x37])?;
        self.command_with_data(0x60, &[0x0C, 0x05])?;
        self.command_with_data(0xE3, &[0xFF])?;
        self.command_with_data(0x84, &[0x00])?;
        Ok(())
    }

    pub fn display_packed(&mut self, packed: &[u8]) -> Result<()> {
        if packed.len() != PACKED_BUFFER_LEN {
            bail!(
                "Waveshare 3inch G packed buffer must be {} bytes, got {}",
                PACKED_BUFFER_LEN,
                packed.len()
            );
        }

        self.command(0x04)?;
        self.wait_busy_high(Duration::from_secs(20))?;
        self.command_with_data(0x10, packed)?;
        self.turn_on_display()
    }

    pub fn clear(&mut self, color: Epd3in0gColor) -> Result<()> {
        self.command(0x04)?;
        self.wait_busy_high(Duration::from_secs(20))?;
        self.command(0x10)?;
        let line = vec![color.repeated_byte(); (PANEL_WIDTH as usize).div_ceil(4)];
        for _ in 0..PANEL_HEIGHT {
            self.data(&line)?;
        }
        self.turn_on_display()
    }

    pub fn sleep(&mut self) -> Result<()> {
        if self.sleeping {
            return Ok(());
        }
        self.command_with_data(0x02, &[0x00])?;
        self.command_with_data(0x07, &[0xA5])?;
        self.sleeping = true;
        Ok(())
    }

    fn reset(&mut self) {
        self.reset.set_high();
        std::thread::sleep(Duration::from_millis(20));
        self.reset.set_low();
        std::thread::sleep(Duration::from_millis(2));
        self.reset.set_high();
        std::thread::sleep(Duration::from_millis(20));
    }

    fn turn_on_display(&mut self) -> Result<()> {
        self.command_with_data(0x12, &[0x00])?;
        self.wait_busy_high(Duration::from_secs(30))?;
        self.command_with_data(0x02, &[0x00])?;
        self.wait_busy_high(Duration::from_secs(20))
    }

    fn command_with_data(&mut self, command: u8, data: &[u8]) -> Result<()> {
        self.command(command)?;
        self.data(data)
    }

    fn command(&mut self, command: u8) -> Result<()> {
        self.dc.set_low();
        self.spi
            .write(&[command])
            .with_context(|| format!("writing e-paper command 0x{command:02X}"))?;
        Ok(())
    }

    fn data(&mut self, data: &[u8]) -> Result<()> {
        self.dc.set_high();
        for chunk in data.chunks(SPI_CHUNK_SIZE) {
            self.spi.write(chunk).context("writing e-paper data")?;
        }
        Ok(())
    }

    fn wait_busy_high(&self, timeout: Duration) -> Result<()> {
        let start = Instant::now();
        while self.busy.is_low() {
            if start.elapsed() > timeout {
                return Err(anyhow!("timed out waiting for e-paper busy pin to go high"));
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        Ok(())
    }
}

#[cfg(target_os = "linux")]
impl Drop for Epd3in0gDevice {
    fn drop(&mut self) {
        let _ = self.sleep();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::ImageBuffer;

    #[test]
    fn packs_four_2bpp_pixels_per_byte() {
        assert_eq!(
            pack_2bpp_pixels(&[
                Epd3in0gColor::Black as u8,
                Epd3in0gColor::White as u8,
                Epd3in0gColor::Yellow as u8,
                Epd3in0gColor::Red as u8,
            ]),
            vec![0b0001_1011]
        );
    }

    #[test]
    fn packs_landscape_frame_to_controller_buffer_size() {
        let frame = ImageBuffer::from_pixel(
            LANDSCAPE_WIDTH as u32,
            LANDSCAPE_HEIGHT as u32,
            Luma([255_u8]),
        );

        let packed = pack_landscape_mono_frame(&frame).unwrap();

        assert_eq!(packed.len(), PACKED_BUFFER_LEN);
        assert!(packed
            .iter()
            .all(|byte| *byte == Epd3in0gColor::White.repeated_byte()));
    }

    #[test]
    fn rotates_landscape_frame_like_waveshare_python_sample() {
        let mut frame = ImageBuffer::from_pixel(
            LANDSCAPE_WIDTH as u32,
            LANDSCAPE_HEIGHT as u32,
            Luma([255_u8]),
        );
        frame.put_pixel(0, 0, Luma([0_u8]));

        let packed = pack_landscape_mono_frame(&frame).unwrap();
        let target_x = 0_usize;
        let target_y = LANDSCAPE_WIDTH as usize - 1;
        let pixel_index = target_y * PANEL_WIDTH as usize + target_x;
        let byte = packed[pixel_index / 4];
        let shift = 6 - ((pixel_index % 4) * 2);

        assert_eq!((byte >> shift) & 0x03, Epd3in0gColor::Black as u8);
    }
}
