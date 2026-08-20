pub const EMPTY_TILE: u8 = 0;
pub const DEFAULT_PLAYER_ANGLE: f32 = -std::f32::consts::FRAC_PI_2;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerStart {
    pub x: f32,
    pub y: f32,
    pub angle: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Goal {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, PartialEq)]
pub struct Level {
    pub tiles: Vec<Vec<u8>>,
    pub player_start: PlayerStart,
    pub goal: Goal,
}

impl Level {
    /// Parsea un mapa rectangular donde `.` es suelo, `P` es el inicio,
    /// `G` es la meta y los caracteres `1` a `9` son tipos de pared.
    pub fn parse(source: &str) -> Result<Self, String> {
        let mut tiles = Vec::new();
        let mut player_start = None;
        let mut goal = None;
        let mut expected_width = None;

        for (row_index, line) in source.lines().enumerate() {
            if line.is_empty() {
                return Err(format!("La fila {} está vacía", row_index + 1));
            }

            let mut row = Vec::with_capacity(line.len());
            for (column_index, tile) in line.bytes().enumerate() {
                match tile {
                    b'.' => row.push(EMPTY_TILE),
                    b'1'..=b'9' => row.push(tile - b'0'),
                    b'P' => {
                        if player_start.is_some() {
                            return Err("El nivel debe tener una única posición inicial".to_owned());
                        }

                        player_start = Some(PlayerStart {
                            x: column_index as f32 + 0.5,
                            y: row_index as f32 + 0.5,
                            angle: DEFAULT_PLAYER_ANGLE,
                        });
                        row.push(EMPTY_TILE);
                    }
                    b'G' => {
                        if goal.is_some() {
                            return Err("El nivel debe tener una única meta".to_owned());
                        }

                        goal = Some(Goal {
                            x: column_index as f32 + 0.5,
                            y: row_index as f32 + 0.5,
                        });
                        row.push(EMPTY_TILE);
                    }
                    _ => {
                        return Err(format!(
                            "Carácter inválido '{}' en fila {}, columna {}",
                            tile as char,
                            row_index + 1,
                            column_index + 1
                        ));
                    }
                }
            }

            if let Some(width) = expected_width {
                if row.len() != width {
                    return Err("El nivel debe ser una grilla rectangular".to_owned());
                }
            } else {
                expected_width = Some(row.len());
            }

            tiles.push(row);
        }

        if tiles.is_empty() {
            return Err("El nivel no puede estar vacío".to_owned());
        }

        let player_start = player_start.ok_or("El nivel no tiene posición inicial 'P'")?;
        let goal = goal.ok_or("El nivel no tiene meta 'G'")?;

        Ok(Self {
            tiles,
            player_start,
            goal,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsea_paredes_y_posicion_inicial() {
        let level = Level::parse("1111\n1PG1\n1331").expect("el mapa debe ser válido");

        assert_eq!(
            level.tiles,
            vec![vec![1, 1, 1, 1], vec![1, 0, 0, 1], vec![1, 3, 3, 1]]
        );
        assert_eq!(
            level.player_start,
            PlayerStart {
                x: 1.5,
                y: 1.5,
                angle: DEFAULT_PLAYER_ANGLE,
            }
        );
        assert_eq!(level.goal, Goal { x: 2.5, y: 1.5 });
    }
}
