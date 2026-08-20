use crate::level::{Level, EMPTY_TILE};
use crate::player::Player;

const CELL_SIZE: usize = 5;
const PADDING: usize = 3;
const SCREEN_MARGIN: usize = 8;
const BACKGROUND_COLOR: u32 = 0x0C0F14;
const GRID_COLOR: u32 = 0x171B22;
const FLOOR_COLOR: u32 = 0x2B3038;
const PLAYER_COLOR: u32 = 0xFFE66D;

/// Overlay compacto con la geometría estática del nivel precalculada.
pub struct Minimap {
    width: usize,
    height: usize,
    static_pixels: Vec<u32>,
}

impl Minimap {
    pub fn new(level: &Level) -> Self {
        let map_width = level.tiles.first().map_or(0, Vec::len);
        let map_height = level.tiles.len();
        let width = map_width * CELL_SIZE + PADDING * 2;
        let height = map_height * CELL_SIZE + PADDING * 2;
        let mut static_pixels = vec![BACKGROUND_COLOR; width * height];

        for (map_y, row) in level.tiles.iter().enumerate() {
            for (map_x, tile) in row.iter().copied().enumerate() {
                let tile_color = if tile == EMPTY_TILE {
                    FLOOR_COLOR
                } else {
                    wall_color(tile)
                };

                for pixel_y in 0..CELL_SIZE {
                    for pixel_x in 0..CELL_SIZE {
                        let color = if pixel_x == 0 || pixel_y == 0 {
                            GRID_COLOR
                        } else {
                            tile_color
                        };
                        let x = PADDING + map_x * CELL_SIZE + pixel_x;
                        let y = PADDING + map_y * CELL_SIZE + pixel_y;
                        static_pixels[y * width + x] = color;
                    }
                }
            }
        }

        Self {
            width,
            height,
            static_pixels,
        }
    }

    pub fn draw(
        &self,
        buffer: &mut [u32],
        screen_width: usize,
        screen_height: usize,
        player: &Player,
    ) {
        if buffer.len() != screen_width * screen_height || screen_width == 0 || screen_height == 0 {
            return;
        }

        let origin_x = screen_width.saturating_sub(self.width + SCREEN_MARGIN);
        let origin_y = SCREEN_MARGIN.min(screen_height);
        let copy_width = self.width.min(screen_width.saturating_sub(origin_x));
        let copy_height = self.height.min(screen_height.saturating_sub(origin_y));

        for row in 0..copy_height {
            let source_start = row * self.width;
            let destination_start = (origin_y + row) * screen_width + origin_x;
            buffer[destination_start..destination_start + copy_width]
                .copy_from_slice(&self.static_pixels[source_start..source_start + copy_width]);
        }

        let player_x = origin_x as i32 + PADDING as i32 + (player.x * CELL_SIZE as f32) as i32;
        let player_y = origin_y as i32 + PADDING as i32 + (player.y * CELL_SIZE as f32) as i32;
        let direction_length = CELL_SIZE as f32 * 2.2;
        let direction_x = player_x + (player.angle.cos() * direction_length).round() as i32;
        let direction_y = player_y + (player.angle.sin() * direction_length).round() as i32;

        draw_line(
            buffer,
            screen_width,
            screen_height,
            player_x,
            player_y,
            direction_x,
            direction_y,
            PLAYER_COLOR,
        );

        for offset_y in -1..=1 {
            for offset_x in -1..=1 {
                put_pixel(
                    buffer,
                    screen_width,
                    screen_height,
                    player_x + offset_x,
                    player_y + offset_y,
                    PLAYER_COLOR,
                );
            }
        }
    }
}

fn wall_color(wall_type: u8) -> u32 {
    match wall_type {
        1 => 0xA33A32,
        2 => 0x4A6E9C,
        3 => 0x4E7838,
        _ => 0x777D88,
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_line(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    mut x0: i32,
    mut y0: i32,
    x1: i32,
    y1: i32,
    color: u32,
) {
    let delta_x = (x1 - x0).abs();
    let step_x = if x0 < x1 { 1 } else { -1 };
    let delta_y = -(y1 - y0).abs();
    let step_y = if y0 < y1 { 1 } else { -1 };
    let mut error = delta_x + delta_y;

    loop {
        put_pixel(buffer, width, height, x0, y0, color);
        if x0 == x1 && y0 == y1 {
            break;
        }
        let double_error = error * 2;
        if double_error >= delta_y {
            error += delta_y;
            x0 += step_x;
        }
        if double_error <= delta_x {
            error += delta_x;
            y0 += step_y;
        }
    }
}

fn put_pixel(buffer: &mut [u32], width: usize, height: usize, x: i32, y: i32, color: u32) {
    let Ok(x) = usize::try_from(x) else {
        return;
    };
    let Ok(y) = usize::try_from(y) else {
        return;
    };

    if x < width && y < height {
        buffer[y * width + x] = color;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dibuja_layout_jugador_y_orientacion_superpuestos() {
        let level = Level::parse("11111\n1P..1\n11111").expect("el nivel debe ser válido");
        let player = Player::from(level.player_start);
        let minimap = Minimap::new(&level);
        let mut buffer = vec![0; 160 * 100];

        minimap.draw(&mut buffer, 160, 100, &player);

        assert!(buffer.contains(&wall_color(1)));
        assert!(buffer.contains(&FLOOR_COLOR));
        assert!(
            buffer
                .iter()
                .filter(|pixel| **pixel == PLAYER_COLOR)
                .count()
                > 9
        );
    }

    #[test]
    fn se_superpone_en_la_esquina_del_framebuffer_real() {
        let level = Level::parse(include_str!("../assets/niveles/prueba.txt"))
            .expect("el nivel debe ser válido");
        let player = Player::from(level.player_start);
        let minimap = Minimap::new(&level);
        let mut buffer = vec![0x202632; 800 * 600];

        minimap.draw(&mut buffer, 800, 600, &player);

        let origin_x = 800 - minimap.width - SCREEN_MARGIN;
        assert_eq!(buffer[SCREEN_MARGIN * 800 + origin_x], BACKGROUND_COLOR);
        assert!(
            buffer
                .iter()
                .filter(|pixel| **pixel == PLAYER_COLOR)
                .count()
                > 9
        );
    }
}
