pub const GLYPH_WIDTH: usize = 5;
pub const GLYPH_HEIGHT: usize = 7;

pub fn draw_centered(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    y: usize,
    text: &[u8],
    scale: usize,
    color: u32,
) {
    let text_width = text
        .len()
        .saturating_mul((GLYPH_WIDTH + 1) * scale)
        .saturating_sub(scale);
    let x = width.saturating_sub(text_width) / 2;
    draw_text(buffer, width, height, x, y, text, scale, color);
}

#[allow(clippy::too_many_arguments)]
pub fn draw_text(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    x: usize,
    y: usize,
    text: &[u8],
    scale: usize,
    color: u32,
) {
    if buffer.len() != width.saturating_mul(height) || scale == 0 {
        return;
    }

    let advance = (GLYPH_WIDTH + 1) * scale;
    for (index, character) in text.iter().enumerate() {
        draw_glyph(
            buffer,
            width,
            height,
            x + index * advance,
            y,
            scale,
            color,
            glyph(*character),
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_glyph(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    origin_x: usize,
    origin_y: usize,
    scale: usize,
    color: u32,
    rows: [u8; GLYPH_HEIGHT],
) {
    for (glyph_y, row) in rows.iter().enumerate() {
        for glyph_x in 0..GLYPH_WIDTH {
            if row & (1 << (GLYPH_WIDTH - glyph_x - 1)) == 0 {
                continue;
            }
            for offset_y in 0..scale {
                let pixel_y = origin_y + glyph_y * scale + offset_y;
                if pixel_y >= height {
                    continue;
                }
                for offset_x in 0..scale {
                    let pixel_x = origin_x + glyph_x * scale + offset_x;
                    if pixel_x < width {
                        buffer[pixel_y * width + pixel_x] = color;
                    }
                }
            }
        }
    }
}

fn glyph(character: u8) -> [u8; GLYPH_HEIGHT] {
    match character {
        b'A' => [14, 17, 17, 31, 17, 17, 17],
        b'B' => [30, 17, 17, 30, 17, 17, 30],
        b'C' => [14, 17, 16, 16, 16, 17, 14],
        b'D' => [30, 17, 17, 17, 17, 17, 30],
        b'E' => [31, 16, 16, 30, 16, 16, 31],
        b'F' => [31, 16, 16, 30, 16, 16, 16],
        b'G' => [14, 17, 16, 23, 17, 17, 15],
        b'H' => [17, 17, 17, 31, 17, 17, 17],
        b'I' => [31, 4, 4, 4, 4, 4, 31],
        b'J' => [7, 2, 2, 2, 18, 18, 12],
        b'K' => [17, 18, 20, 24, 20, 18, 17],
        b'L' => [16, 16, 16, 16, 16, 16, 31],
        b'M' => [17, 27, 21, 21, 17, 17, 17],
        b'N' => [17, 25, 21, 19, 17, 17, 17],
        b'O' => [14, 17, 17, 17, 17, 17, 14],
        b'P' => [30, 17, 17, 30, 16, 16, 16],
        b'Q' => [14, 17, 17, 17, 21, 18, 13],
        b'R' => [30, 17, 17, 30, 20, 18, 17],
        b'S' => [15, 16, 16, 14, 1, 1, 30],
        b'T' => [31, 4, 4, 4, 4, 4, 4],
        b'U' => [17, 17, 17, 17, 17, 17, 14],
        b'V' => [17, 17, 17, 17, 17, 10, 4],
        b'W' => [17, 17, 17, 21, 21, 21, 10],
        b'X' => [17, 17, 10, 4, 10, 17, 17],
        b'Y' => [17, 17, 10, 4, 4, 4, 4],
        b'Z' => [31, 1, 2, 4, 8, 16, 31],
        b'0' => [14, 17, 19, 21, 25, 17, 14],
        b'1' => [4, 12, 4, 4, 4, 4, 14],
        b'2' => [14, 17, 1, 2, 4, 8, 31],
        b'3' => [30, 1, 1, 14, 1, 1, 30],
        b'4' => [2, 6, 10, 18, 31, 2, 2],
        b'5' => [31, 16, 16, 30, 1, 1, 30],
        b'6' => [14, 16, 16, 30, 17, 17, 14],
        b'7' => [31, 1, 2, 4, 8, 8, 8],
        b'8' => [14, 17, 17, 14, 17, 17, 14],
        b'9' => [14, 17, 17, 15, 1, 1, 14],
        b':' => [0, 4, 4, 0, 4, 4, 0],
        b'/' => [1, 1, 2, 4, 8, 16, 16],
        b'-' => [0, 0, 0, 31, 0, 0, 0],
        _ => [0; GLYPH_HEIGHT],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn centra_y_dibuja_texto() {
        let mut buffer = vec![0; 100 * 30];
        draw_centered(&mut buffer, 100, 30, 4, b"MENU", 2, 0xFFFFFF);
        assert!(buffer.contains(&0xFFFFFF));
    }
}
