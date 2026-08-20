use std::time::{Duration, Instant};

const SAMPLE_INTERVAL: Duration = Duration::from_millis(500);
const GLYPH_WIDTH: usize = 5;
const GLYPH_HEIGHT: usize = 7;
const GLYPH_SCALE: usize = 2;
const BACKGROUND_COLOR: u32 = 0x101318;
const TEXT_COLOR: u32 = 0xF4F4E8;

pub struct FpsCounter {
    sample_started: Instant,
    frames: u32,
    displayed_fps: u32,
}

impl FpsCounter {
    pub fn new() -> Self {
        Self {
            sample_started: Instant::now(),
            frames: 0,
            displayed_fps: 0,
        }
    }

    pub fn frame_rendered(&mut self) {
        self.frames = self.frames.saturating_add(1);
        let elapsed = self.sample_started.elapsed();

        if elapsed >= SAMPLE_INTERVAL {
            self.displayed_fps = (self.frames as f32 / elapsed.as_secs_f32()).round() as u32;
            self.frames = 0;
            self.sample_started = Instant::now();
        }
    }

    pub fn draw(&self, buffer: &mut [u32], width: usize, height: usize) {
        if buffer.len() != width * height || width == 0 || height == 0 {
            return;
        }

        let origin_x = 6;
        let origin_y = 6;
        let advance = (GLYPH_WIDTH + 1) * GLYPH_SCALE;
        let overlay_width = origin_x + advance * 7;
        let overlay_height = origin_y + GLYPH_HEIGHT * GLYPH_SCALE + 4;

        for row in origin_y.saturating_sub(3)..overlay_height.min(height) {
            let start = row * width + origin_x.saturating_sub(3).min(width);
            let end = (row * width + overlay_width.min(width)).max(start);
            buffer[start..end].fill(BACKGROUND_COLOR);
        }

        let fps = self.displayed_fps.min(999);
        let glyphs = [
            glyph_for(b'F'),
            glyph_for(b'P'),
            glyph_for(b'S'),
            glyph_for(b' '),
            digit_glyph((fps / 100) as u8),
            digit_glyph(((fps / 10) % 10) as u8),
            digit_glyph((fps % 10) as u8),
        ];

        for (index, glyph) in glyphs.iter().enumerate() {
            draw_glyph(
                buffer,
                width,
                height,
                origin_x + index * advance,
                origin_y,
                glyph,
            );
        }
    }
}

fn draw_glyph(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    origin_x: usize,
    origin_y: usize,
    glyph: &[u8; GLYPH_HEIGHT],
) {
    for (glyph_y, row_bits) in glyph.iter().enumerate() {
        for glyph_x in 0..GLYPH_WIDTH {
            if row_bits & (1 << (GLYPH_WIDTH - 1 - glyph_x)) == 0 {
                continue;
            }

            for scale_y in 0..GLYPH_SCALE {
                let y = origin_y + glyph_y * GLYPH_SCALE + scale_y;
                if y >= height {
                    continue;
                }
                for scale_x in 0..GLYPH_SCALE {
                    let x = origin_x + glyph_x * GLYPH_SCALE + scale_x;
                    if x < width {
                        buffer[y * width + x] = TEXT_COLOR;
                    }
                }
            }
        }
    }
}

fn glyph_for(character: u8) -> [u8; GLYPH_HEIGHT] {
    match character {
        b'F' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000,
        ],
        b'P' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000,
        ],
        b'S' => [
            0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110,
        ],
        _ => [0; GLYPH_HEIGHT],
    }
}

fn digit_glyph(digit: u8) -> [u8; GLYPH_HEIGHT] {
    const DIGITS: [[u8; GLYPH_HEIGHT]; 10] = [
        [
            0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110,
        ],
        [
            0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110,
        ],
        [
            0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111,
        ],
        [
            0b11110, 0b00001, 0b00001, 0b01110, 0b00001, 0b00001, 0b11110,
        ],
        [
            0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010,
        ],
        [
            0b11111, 0b10000, 0b10000, 0b11110, 0b00001, 0b00001, 0b11110,
        ],
        [
            0b01110, 0b10000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110,
        ],
        [
            0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000,
        ],
        [
            0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110,
        ],
        [
            0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00001, 0b01110,
        ],
    ];

    DIGITS[digit.min(9) as usize]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dibuja_el_contador_sin_salirse_del_buffer() {
        let mut counter = FpsCounter::new();
        counter.displayed_fps = 60;
        let mut buffer = vec![0; 120 * 30];

        counter.draw(&mut buffer, 120, 30);

        assert!(buffer.contains(&TEXT_COLOR));
        assert!(buffer.contains(&BACKGROUND_COLOR));
    }
}
