use crate::level::{Level, PlayerStart, EMPTY_TILE};

pub const DEFAULT_FOV: f32 = std::f32::consts::FRAC_PI_3;

const CEILING_COLOR: u32 = 0x202632;
const FLOOR_COLOR: u32 = 0x38332D;
const MIN_DISTANCE: f32 = 0.0001;
const DDA_EPSILON: f32 = 0.000001;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HitSide {
    Vertical,
    Horizontal,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RayHit {
    /// Distancia perpendicular al plano de cámara. Ya está corregida para fisheye.
    pub distance: f32,
    pub wall_type: u8,
    pub side: HitSide,
}

pub fn render(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    level: &Level,
    player: PlayerStart,
    fov: f32,
) {
    assert_eq!(buffer.len(), width * height);

    let horizon = height / 2;
    buffer[..horizon * width].fill(CEILING_COLOR);
    buffer[horizon * width..].fill(FLOOR_COLOR);

    if width == 0 || height == 0 {
        return;
    }

    let fov = if fov.is_finite() {
        fov.clamp(1.0_f32.to_radians(), 179.0_f32.to_radians())
    } else {
        DEFAULT_FOV
    };
    let direction_x = player.angle.cos();
    let direction_y = player.angle.sin();
    let camera_plane_scale = (fov * 0.5).tan();
    let plane_x = -direction_y * camera_plane_scale;
    let plane_y = direction_x * camera_plane_scale;
    let focal_length = width as f32 / (2.0 * camera_plane_scale);

    for column in 0..width {
        let camera_x = if width == 1 {
            0.0
        } else {
            2.0 * column as f32 / (width - 1) as f32 - 1.0
        };
        let ray_direction_x = direction_x + plane_x * camera_x;
        let ray_direction_y = direction_y + plane_y * camera_x;

        // Como el plano es perpendicular a una dirección unitaria, el parámetro
        // que devuelve el DDA equivale a la distancia perpendicular y evita fisheye.
        let Some(hit) = cast_ray(level, player.x, player.y, ray_direction_x, ray_direction_y)
        else {
            continue;
        };

        let wall_height = (focal_length / hit.distance.max(MIN_DISTANCE)) as usize;
        let draw_start = horizon.saturating_sub(wall_height / 2);
        let draw_end = (horizon + wall_height.div_ceil(2)).min(height);
        let color = wall_color(hit.wall_type, hit.side);

        for row in draw_start..draw_end {
            buffer[row * width + column] = color;
        }
    }
}

pub fn cast_ray(
    level: &Level,
    origin_x: f32,
    origin_y: f32,
    direction_x: f32,
    direction_y: f32,
) -> Option<RayHit> {
    if !origin_x.is_finite()
        || !origin_y.is_finite()
        || !direction_x.is_finite()
        || !direction_y.is_finite()
        || (direction_x.abs() < DDA_EPSILON && direction_y.abs() < DDA_EPSILON)
    {
        return None;
    }

    let mut map_x = origin_x.floor() as i32;
    let mut map_y = origin_y.floor() as i32;
    tile_at(level, map_x, map_y)?;

    let delta_distance_x = reciprocal_or_infinity(direction_x);
    let delta_distance_y = reciprocal_or_infinity(direction_y);
    let (step_x, mut side_distance_x) =
        initial_step_and_distance(origin_x, map_x, direction_x, delta_distance_x);
    let (step_y, mut side_distance_y) =
        initial_step_and_distance(origin_y, map_y, direction_y, delta_distance_y);

    let max_steps = level.tiles.len() * level.tiles.first()?.len() * 2 + 1;

    for _ in 0..max_steps {
        let tolerance = DDA_EPSILON * side_distance_x.abs().max(side_distance_y.abs()).max(1.0);

        let crosses_exact_corner = side_distance_x.is_finite()
            && side_distance_y.is_finite()
            && (side_distance_x - side_distance_y).abs() <= tolerance;

        if crosses_exact_corner {
            let distance = side_distance_x.min(side_distance_y);
            let next_x = map_x + step_x;
            let next_y = map_y + step_y;

            let x_wall = tile_at(level, next_x, map_y).filter(|tile| *tile != EMPTY_TILE);
            let y_wall = tile_at(level, map_x, next_y).filter(|tile| *tile != EMPTY_TILE);

            if let Some(wall_type) = x_wall {
                return Some(RayHit {
                    distance,
                    wall_type,
                    side: HitSide::Vertical,
                });
            }
            if let Some(wall_type) = y_wall {
                return Some(RayHit {
                    distance,
                    wall_type,
                    side: HitSide::Horizontal,
                });
            }

            map_x = next_x;
            map_y = next_y;
            side_distance_x += delta_distance_x;
            side_distance_y += delta_distance_y;

            let wall_type = tile_at(level, map_x, map_y)?;
            if wall_type != EMPTY_TILE {
                let side = if direction_x.abs() >= direction_y.abs() {
                    HitSide::Vertical
                } else {
                    HitSide::Horizontal
                };
                return Some(RayHit {
                    distance,
                    wall_type,
                    side,
                });
            }
        } else if side_distance_x < side_distance_y {
            map_x += step_x;
            let distance = side_distance_x;
            side_distance_x += delta_distance_x;

            let wall_type = tile_at(level, map_x, map_y)?;
            if wall_type != EMPTY_TILE {
                return Some(RayHit {
                    distance,
                    wall_type,
                    side: HitSide::Vertical,
                });
            }
        } else {
            map_y += step_y;
            let distance = side_distance_y;
            side_distance_y += delta_distance_y;

            let wall_type = tile_at(level, map_x, map_y)?;
            if wall_type != EMPTY_TILE {
                return Some(RayHit {
                    distance,
                    wall_type,
                    side: HitSide::Horizontal,
                });
            }
        }
    }

    None
}

fn tile_at(level: &Level, x: i32, y: i32) -> Option<u8> {
    let x = usize::try_from(x).ok()?;
    let y = usize::try_from(y).ok()?;
    level.tiles.get(y)?.get(x).copied()
}

fn reciprocal_or_infinity(direction: f32) -> f32 {
    if direction.abs() < DDA_EPSILON {
        f32::INFINITY
    } else {
        direction.recip().abs()
    }
}

fn initial_step_and_distance(
    origin: f32,
    map_coordinate: i32,
    direction: f32,
    delta_distance: f32,
) -> (i32, f32) {
    if direction < 0.0 {
        (-1, (origin - map_coordinate as f32) * delta_distance)
    } else {
        (1, (map_coordinate as f32 + 1.0 - origin) * delta_distance)
    }
}

fn wall_color(wall_type: u8, side: HitSide) -> u32 {
    let color = match wall_type {
        1 => 0xD95757,
        2 => 0x4B75D1,
        3 => 0x4FAE68,
        4 => 0xD6A84B,
        5 => 0xA965C4,
        6 => 0x45B8B0,
        7 => 0xD97945,
        8 => 0x8A96A8,
        9 => 0xD96E9F,
        _ => 0xFFFFFF,
    };

    match side {
        HitSide::Vertical => color,
        HitSide::Horizontal => shade(color, 0.72),
    }
}

fn shade(color: u32, factor: f32) -> u32 {
    let red = (((color >> 16) & 0xFF) as f32 * factor) as u32;
    let green = (((color >> 8) & 0xFF) as f32 * factor) as u32;
    let blue = ((color & 0xFF) as f32 * factor) as u32;
    (red << 16) | (green << 8) | blue
}

#[cfg(test)]
mod tests {
    use super::*;

    fn open_room() -> Level {
        Level::parse("1111111\n1.....1\n1P....1\n1.....1\n1111111")
            .expect("el mapa debe ser válido")
    }

    #[test]
    fn maneja_rayos_paralelos_a_los_ejes() {
        let level = open_room();

        let east = cast_ray(&level, 1.5, 2.5, 1.0, 0.0).expect("debe tocar la pared este");
        let north = cast_ray(&level, 1.5, 2.5, 0.0, -1.0).expect("debe tocar la pared norte");

        assert!((east.distance - 4.5).abs() < DDA_EPSILON);
        assert!((north.distance - 1.5).abs() < DDA_EPSILON);
    }

    #[test]
    fn mantiene_distancia_perpendicular_sin_fisheye() {
        let level = open_room();

        for direction_y in [-0.2, 0.0, 0.2] {
            let hit = cast_ray(&level, 1.5, 2.5, 1.0, direction_y)
                .expect("el rayo debe tocar la misma pared");
            assert!((hit.distance - 4.5).abs() < DDA_EPSILON);
        }
    }

    #[test]
    fn detecta_una_pared_justo_en_una_esquina() {
        let level = Level::parse("1111\n1P.1\n1.21\n1111").expect("el mapa debe ser válido");

        let hit = cast_ray(&level, 1.5, 1.5, 1.0, 1.0)
            .expect("el rayo debe tocar la esquina de la pared");

        assert!((hit.distance - 0.5).abs() < DDA_EPSILON);
        assert_eq!(hit.wall_type, 2);
    }

    #[test]
    fn renderiza_los_tres_tipos_del_nivel_de_prueba() {
        let level = Level::parse(include_str!("../assets/niveles/prueba.txt"))
            .expect("el nivel de prueba debe ser válido");
        let mut buffer = vec![0; 320 * 200];

        render(
            &mut buffer,
            320,
            200,
            &level,
            level.player_start,
            DEFAULT_FOV,
        );

        for wall_type in 1..=3 {
            let front_color = wall_color(wall_type, HitSide::Vertical);
            let side_color = wall_color(wall_type, HitSide::Horizontal);
            assert!(
                buffer.contains(&front_color) || buffer.contains(&side_color),
                "debe verse la pared tipo {wall_type}"
            );
        }
    }
}
