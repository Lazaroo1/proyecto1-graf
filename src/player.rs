use crate::level::{Level, PlayerStart, EMPTY_TILE};

pub const COLLISION_RADIUS: f32 = 0.2;

const SWEEP_STEP: f32 = COLLISION_RADIUS * 0.5;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Player {
    pub x: f32,
    pub y: f32,
    pub angle: f32,
}

impl From<PlayerStart> for Player {
    fn from(start: PlayerStart) -> Self {
        Self {
            x: start.x,
            y: start.y,
            angle: start.angle,
        }
    }
}

impl Player {
    /// Mueve al jugador barriendo todo el trayecto y deslizando por cada eje.
    /// Devuelve `true` si al menos una coordenada pudo cambiar.
    pub fn move_by(&mut self, level: &Level, delta_x: f32, delta_y: f32) -> bool {
        if !delta_x.is_finite()
            || !delta_y.is_finite()
            || circle_collides(level, self.x, self.y, COLLISION_RADIUS)
        {
            return false;
        }

        let requested_distance = delta_x.hypot(delta_y);
        if requested_distance == 0.0 {
            return false;
        }

        // No hace falta barrer más que el tamaño total del mapa: cualquier
        // recorrido mayor ya habría encontrado el borde, que también es sólido.
        let map_width = level.tiles.first().map_or(0, Vec::len);
        let max_distance = (map_width + level.tiles.len() + 1) as f32;
        let scale = (max_distance / requested_distance).min(1.0);
        let delta_x = delta_x * scale;
        let delta_y = delta_y * scale;
        let distance = requested_distance * scale;
        let steps = (distance / SWEEP_STEP).ceil().max(1.0) as usize;
        let step_x = delta_x / steps as f32;
        let step_y = delta_y / steps as f32;
        let original_x = self.x;
        let original_y = self.y;

        for _ in 0..steps {
            let candidate_x = self.x + step_x;
            if !circle_collides(level, candidate_x, self.y, COLLISION_RADIUS) {
                self.x = candidate_x;
            }

            let candidate_y = self.y + step_y;
            if !circle_collides(level, self.x, candidate_y, COLLISION_RADIUS) {
                self.y = candidate_y;
            }
        }

        debug_assert!(!circle_collides(level, self.x, self.y, COLLISION_RADIUS));

        self.x != original_x || self.y != original_y
    }
}

fn circle_collides(level: &Level, center_x: f32, center_y: f32, radius: f32) -> bool {
    if !center_x.is_finite() || !center_y.is_finite() || !radius.is_finite() || radius <= 0.0 {
        return true;
    }

    let min_x = (center_x - radius).floor() as i32;
    let max_x = (center_x + radius).floor() as i32;
    let min_y = (center_y - radius).floor() as i32;
    let max_y = (center_y + radius).floor() as i32;
    let radius_squared = radius * radius;

    for tile_y in min_y..=max_y {
        for tile_x in min_x..=max_x {
            let Some(tile) = tile_at(level, tile_x, tile_y) else {
                return true;
            };

            if tile == EMPTY_TILE {
                continue;
            }

            let closest_x = center_x.clamp(tile_x as f32, tile_x as f32 + 1.0);
            let closest_y = center_y.clamp(tile_y as f32, tile_y as f32 + 1.0);
            let distance_x = center_x - closest_x;
            let distance_y = center_y - closest_y;

            if distance_x * distance_x + distance_y * distance_y < radius_squared {
                return true;
            }
        }
    }

    false
}

fn tile_at(level: &Level, x: i32, y: i32) -> Option<u8> {
    let x = usize::try_from(x).ok()?;
    let y = usize::try_from(y).ok()?;
    level.tiles.get(y)?.get(x).copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 0.0001;

    #[test]
    fn empujar_repetidamente_una_pared_respeta_el_radio() {
        let level = Level::parse("11111\n1P.G1\n11111").expect("el mapa debe ser válido");
        let mut player = Player::from(level.player_start);

        for _ in 0..100 {
            player.move_by(&level, -0.25, 0.0);
        }
        let position_against_wall = player.x;

        for _ in 0..100 {
            player.move_by(&level, -0.25, 0.0);
        }

        assert!(player.x >= 1.0 + COLLISION_RADIUS - EPSILON);
        assert!((player.x - position_against_wall).abs() < EPSILON);
        assert!((player.y - 1.5).abs() < EPSILON);
    }

    #[test]
    fn un_movimiento_grande_no_atraviesa_una_pared() {
        let level = Level::parse("1111111\n1P.1.G1\n1111111").expect("el mapa debe ser válido");
        let mut player = Player::from(level.player_start);

        player.move_by(&level, 20.0, 0.0);

        assert!(player.x <= 3.0 - COLLISION_RADIUS + EPSILON);
        assert!(player.x >= 3.0 - COLLISION_RADIUS - SWEEP_STEP - EPSILON);
        assert!(!circle_collides(
            &level,
            player.x,
            player.y,
            COLLISION_RADIUS
        ));
    }

    #[test]
    fn el_movimiento_diagonal_no_se_cuela_por_una_esquina() {
        let level = Level::parse("1111\n1P11\n1111\n111G").expect("el mapa debe ser válido");
        let mut player = Player::from(level.player_start);

        for _ in 0..100 {
            player.move_by(&level, 0.5, 0.5);
        }

        assert!(player.x <= 2.0 - COLLISION_RADIUS + EPSILON);
        assert!(player.y <= 2.0 - COLLISION_RADIUS + EPSILON);
        assert!(!circle_collides(
            &level,
            player.x,
            player.y,
            COLLISION_RADIUS
        ));
    }
}
