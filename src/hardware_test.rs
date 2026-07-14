use anyhow::Result;
use clap::ValueEnum;
use image::{ImageBuffer, Luma};

use crate::render::MonoFrame;
use crate::waveshare3ing::{
    new_landscape_color_frame, ColorFrame, Epd3in0gColor, LANDSCAPE_HEIGHT, LANDSCAPE_WIDTH,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum HardwareTestPattern {
    White,
    Black,
    Border,
    Checkerboard,
    Text,
    Red,
    Yellow,
    ColorQuadrants,
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
        HardwareTestPattern::Red
        | HardwareTestPattern::Yellow
        | HardwareTestPattern::ColorQuadrants => {}
    }

    frame
}

pub fn run_hardware_test(pattern: HardwareTestPattern) -> Result<()> {
    match pattern {
        HardwareTestPattern::Red
        | HardwareTestPattern::Yellow
        | HardwareTestPattern::ColorQuadrants => {
            let frame = color_pattern_frame(pattern);
            run_waveshare_color_frame(&frame)
        }
        _ => {
            let frame = pattern_frame(pattern);
            run_waveshare_frame(&frame)
        }
    }
}

pub fn color_pattern_frame(pattern: HardwareTestPattern) -> ColorFrame {
    match pattern {
        HardwareTestPattern::Red => new_landscape_color_frame(Epd3in0gColor::Red),
        HardwareTestPattern::Yellow => new_landscape_color_frame(Epd3in0gColor::Yellow),
        HardwareTestPattern::ColorQuadrants => draw_color_quadrants(),
        _ => new_landscape_color_frame(Epd3in0gColor::White),
    }
}

#[cfg(target_os = "linux")]
fn run_waveshare_frame(frame: &MonoFrame) -> Result<()> {
    let packed = crate::waveshare3ing::pack_landscape_mono_frame(frame)?;
    let mut device = crate::waveshare3ing::Epd3in0gDevice::open_default()?;
    device.init()?;
    device.display_packed(&packed)
}

#[cfg(target_os = "linux")]
fn run_waveshare_color_frame(frame: &[Epd3in0gColor]) -> Result<()> {
    let packed = crate::waveshare3ing::pack_landscape_color_frame(frame)?;
    let mut device = crate::waveshare3ing::Epd3in0gDevice::open_default()?;
    device.init()?;
    device.display_packed(&packed)
}

#[cfg(not(target_os = "linux"))]
fn run_waveshare_frame(_frame: &MonoFrame) -> Result<()> {
    anyhow::bail!("hardware tests require Linux/Raspberry Pi")
}

#[cfg(not(target_os = "linux"))]
fn run_waveshare_color_frame(_frame: &[Epd3in0gColor]) -> Result<()> {
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

fn draw_color_quadrants() -> ColorFrame {
    let mut frame = new_landscape_color_frame(Epd3in0gColor::White);
    for y in 0..LANDSCAPE_HEIGHT as usize {
        for x in 0..LANDSCAPE_WIDTH as usize {
            let color = match (
                x < LANDSCAPE_WIDTH as usize / 2,
                y < LANDSCAPE_HEIGHT as usize / 2,
            ) {
                (true, true) => Epd3in0gColor::Black,
                (false, true) => Epd3in0gColor::White,
                (true, false) => Epd3in0gColor::Red,
                (false, false) => Epd3in0gColor::Yellow,
            };
            frame[y * LANDSCAPE_WIDTH as usize + x] = color;
        }
    }
    frame
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

    #[test]
    fn color_patterns_have_panel_landscape_dimensions() {
        for pattern in [
            HardwareTestPattern::Red,
            HardwareTestPattern::Yellow,
            HardwareTestPattern::ColorQuadrants,
        ] {
            let frame = color_pattern_frame(pattern);
            assert_eq!(
                frame.len(),
                LANDSCAPE_WIDTH as usize * LANDSCAPE_HEIGHT as usize
            );
        }
    }

    #[test]
    fn color_quadrants_cover_all_panel_colors() {
        let frame = color_pattern_frame(HardwareTestPattern::ColorQuadrants);

        assert!(frame.contains(&Epd3in0gColor::Black));
        assert!(frame.contains(&Epd3in0gColor::White));
        assert!(frame.contains(&Epd3in0gColor::Red));
        assert!(frame.contains(&Epd3in0gColor::Yellow));
    }
}
