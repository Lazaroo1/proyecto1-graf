use crate::level::{Level, EMPTY_TILE};
use crate::player::Player;
use crate::texture::WallTextures;

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
    pub texture_u: f32,
}

pub struct Raycaster {
    width: usize,
    height: usize,
    horizon: usize,
    camera_plane_scale: f32,
    focal_length: f32,
    camera_x_by_column: Vec<f32>,
}

impl Raycaster {
    pub fn new(width: usize, height: usize, fov: f32) -> Self {
        assert!(width > 0 && height > 0);

        let fov = if fov.is_finite() {
            fov.clamp(1.0_f32.to_radians(), 179.0_f32.to_radians())
        } else {
            DEFAULT_FOV
        };
        let camera_plane_scale = (fov * 0.5).tan();
        let camera_x_by_column = (0..width)
            .map(|column| {
                if width == 1 {
                    0.0
                } else {
                    2.0 * column as f32 / (width - 1) as f32 - 1.0
                }
            })
            .collect();

        Self {
            width,
            height,
            horizon: height / 2,
            camera_plane_scale,
            focal_length: width as f32 / (2.0 * camera_plane_scale),
            camera_x_by_column,
        }
    }

    pub fn render(
        &self,
        buffer: &mut [u32],
        level: &Level,
        textures: &WallTextures,
        player: &Player,
    ) {
        assert_eq!(buffer.len(), self.width * self.height);

        buffer[..self.horizon * self.width].fill(CEILING_COLOR);
        buffer[self.horizon * self.width..].fill(FLOOR_COLOR);

        let direction_x = player.angle.cos();
        let direction_y = player.angle.sin();
        let plane_x = -direction_y * self.camera_plane_scale;
        let plane_y = direction_x * self.camera_plane_scale;

        for (column, camera_x) in self.camera_x_by_column.iter().copied().enumerate() {
            let ray_direction_x = direction_x + plane_x * camera_x;
            let ray_direction_y = direction_y + plane_y * camera_x;
            let Some(hit) = cast_ray(level, player.x, player.y, ray_direction_x, ray_direction_y)
            else {
                continue;
            };

            let wall_height = self.focal_length / hit.distance.max(MIN_DISTANCE);
            let wall_top = self.horizon as f32 - wall_height * 0.5;
            let wall_bottom = self.horizon as f32 + wall_height * 0.5;
            let draw_start = wall_top.floor().max(0.0) as usize;
            let draw_end = wall_bottom.ceil().min(self.height as f32) as usize;
            let texture_column = textures.column(
                hit.wall_type,
                hit.texture_u,
                hit.side == HitSide::Horizontal,
            );
            let texture_step = wall_height.recip();
            let mut texture_v = (draw_start as f32 + 0.5 - wall_top) * texture_step;

            for row in draw_start..draw_end {
                buffer[row * self.width + column] = texture_column.sample(texture_v);
                texture_v += texture_step;
            }
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
                return Some(make_hit(
                    distance,
                    wall_type,
                    HitSide::Vertical,
                    origin_x,
                    origin_y,
                    direction_x,
                    direction_y,
                ));
            }
            if let Some(wall_type) = y_wall {
                return Some(make_hit(
                    distance,
                    wall_type,
                    HitSide::Horizontal,
                    origin_x,
                    origin_y,
                    direction_x,
                    direction_y,
                ));
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
                return Some(make_hit(
                    distance,
                    wall_type,
                    side,
                    origin_x,
                    origin_y,
                    direction_x,
                    direction_y,
                ));
            }
        } else if side_distance_x < side_distance_y {
            map_x += step_x;
            let distance = side_distance_x;
            side_distance_x += delta_distance_x;

            let wall_type = tile_at(level, map_x, map_y)?;
            if wall_type != EMPTY_TILE {
                return Some(make_hit(
                    distance,
                    wall_type,
                    HitSide::Vertical,
                    origin_x,
                    origin_y,
                    direction_x,
                    direction_y,
                ));
            }
        } else {
            map_y += step_y;
            let distance = side_distance_y;
            side_distance_y += delta_distance_y;

            let wall_type = tile_at(level, map_x, map_y)?;
            if wall_type != EMPTY_TILE {
                return Some(make_hit(
                    distance,
                    wall_type,
                    HitSide::Horizontal,
                    origin_x,
                    origin_y,
                    direction_x,
                    direction_y,
                ));
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

fn make_hit(
    distance: f32,
    wall_type: u8,
    side: HitSide,
    origin_x: f32,
    origin_y: f32,
    direction_x: f32,
    direction_y: f32,
) -> RayHit {
    let wall_coordinate = match side {
        HitSide::Vertical => origin_y + distance * direction_y,
        HitSide::Horizontal => origin_x + distance * direction_x,
    };
    let mut texture_u = wall_coordinate.rem_euclid(1.0);

    if (side == HitSide::Vertical && direction_x > 0.0)
        || (side == HitSide::Horizontal && direction_y < 0.0)
    {
        texture_u = (1.0 - texture_u).rem_euclid(1.0);
    }

    RayHit {
        distance,
        wall_type,
        side,
        texture_u,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn open_room() -> Level {
        Level::parse("1111111\n1....G1\n1P....1\n1.....1\n1111111")
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
    fn calcula_u_desde_el_punto_exacto_de_impacto() {
        let level = open_room();

        let hit = cast_ray(&level, 1.5, 2.25, 1.0, 0.0).expect("debe tocar la pared este");

        assert!((hit.texture_u - 0.75).abs() < DDA_EPSILON);
    }

    #[test]
    fn detecta_una_pared_justo_en_una_esquina() {
        let level = Level::parse("1111\n1PG1\n1.21\n1111").expect("el mapa debe ser válido");

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
        let player = Player::from(level.player_start);
        let textures = WallTextures::load_embedded().expect("las texturas deben cargar");
        let raycaster = Raycaster::new(320, 200, DEFAULT_FOV);

        raycaster.render(&mut buffer, &level, &textures, &player);

        let unique_colors = buffer
            .iter()
            .copied()
            .collect::<std::collections::HashSet<_>>();
        assert!(unique_colors.len() > 32);
    }
}
