use anyhow::Result;
use clap::ValueEnum;
use image::{ImageBuffer, Luma};

use crate::render::MonoFrame;
use crate::waveshare3ing::{LANDSCAPE_HEIGHT, LANDSCAPE_WIDTH};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum HardwareTestPattern {
    White,
    Black,
    Border,
    Checkerboard,
    Text,
}

pub fn pattern_frame(pattern: HardwareTestPattern) -> MonoFrame {
    let mut frame = ImageBuffer::from_pixel(
        LANDSCAPE_WIDTH as u32,
        LANDSCAPE_HEIGHT as u32,
        Luma([255_u8]),
    );

    match pattern {
        HardwareTestPattern::White => {}
        HardwareTestPattern::Black => fill(&mut frame, 0),
        HardwareTestPattern::Border => draw_border(&mut frame),
        HardwareTestPattern::Checkerboard => draw_checkerboard(&mut frame),
        HardwareTestPattern::Text => draw_text_pattern(&mut frame),
    }

    frame
}

pub fn run_hardware_test(pattern: HardwareTestPattern) -> Result<()> {
    let frame = pattern_frame(pattern);
    run_waveshare_frame(&frame)
}

#[cfg(target_os = "linux")]
fn run_waveshare_frame(frame: &MonoFrame) -> Result<()> {
    let packed = crate::waveshare3ing::pack_landscape_mono_frame(frame)?;
    let mut device = crate::waveshare3ing::Epd3in0gDevice::open_default()?;
    device.init()?;
    device.display_packed(&packed)
}

#[cfg(not(target_os = "linux"))]
fn run_waveshare_frame(_frame: &MonoFrame) -> Result<()> {
    anyhow::bail!("hardware tests require Linux/Raspberry Pi")
}

fn fill(frame: &mut MonoFrame, value: u8) {
    for pixel in frame.pixels_mut() {
        *pixel = Luma([value]);
    }
}

fn draw_border(frame: &mut MonoFrame) {
    let width = frame.width();
    let height = frame.height();
    for x in 0..width {
        frame.put_pixel(x, 0, Luma([0]));
        frame.put_pixel(x, height - 1, Luma([0]));
    }
    for y in 0..height {
        frame.put_pixel(0, y, Luma([0]));
        frame.put_pixel(width - 1, y, Luma([0]));
    }
}

fn draw_checkerboard(frame: &mut MonoFrame) {
    for y in 0..frame.height() {
        for x in 0..frame.width() {
            let black = ((x / 16) + (y / 16)) % 2 == 0;
            frame.put_pixel(x, y, Luma([if black { 0 } else { 255 }]));
        }
    }
}

fn draw_text_pattern(frame: &mut MonoFrame) {
    draw_border(frame);
    draw_block_text(frame, 24, 28, "TERMPAPER");
    draw_block_text(frame, 24, 76, "WAVESHARE 3IN G");
    draw_block_text(frame, 24, 124, "400X168 FULL REFRESH");
}

fn draw_block_text(frame: &mut MonoFrame, x: u32, y: u32, text: &str) {
    for (index, ch) in text.chars().enumerate() {
        draw_block_char(frame, x + index as u32 * 14, y, ch);
    }
}

fn draw_block_char(frame: &mut MonoFrame, x: u32, y: u32, ch: char) {
    let code = ch as u8;
    for row in 0..9 {
        for col in 0..7 {
            let on = row == 0
                || row == 8
                || col == 0
                || col == 6
                || ((code >> ((row + col) % 8)) & 1) == 1;
            if on {
                for yy in 0..2 {
                    for xx in 0..2 {
                        let px = x + col * 2 + xx;
                        let py = y + row * 2 + yy;
                        if px < frame.width() && py < frame.height() {
                            frame.put_pixel(px, py, Luma([0]));
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn patterns_have_panel_landscape_dimensions() {
        for pattern in [
            HardwareTestPattern::White,
            HardwareTestPattern::Black,
            HardwareTestPattern::Border,
            HardwareTestPattern::Checkerboard,
            HardwareTestPattern::Text,
        ] {
            let frame = pattern_frame(pattern);
            assert_eq!(frame.width(), LANDSCAPE_WIDTH as u32);
            assert_eq!(frame.height(), LANDSCAPE_HEIGHT as u32);
        }
    }

    #[test]
    fn black_pattern_is_all_black() {
        let frame = pattern_frame(HardwareTestPattern::Black);
        assert!(frame.pixels().all(|pixel| pixel[0] == 0));
    }
}
