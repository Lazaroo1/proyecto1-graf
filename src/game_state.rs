use crate::{bitmap_font, level::Goal, player::Player};

const GOAL_RADIUS: f32 = 0.4;
const MENU_BACKGROUND: u32 = 0x111827;
const SUCCESS_BACKGROUND: u32 = 0x102A1D;
const TITLE_COLOR: u32 = 0xF4D35E;
const TEXT_COLOR: u32 = 0xF7FAFC;
const ACCENT_COLOR: u32 = 0x62C3ED;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    Menu,
    Playing,
    Success,
}

impl GameState {
    pub fn start(&mut self) {
        if *self == Self::Menu {
            *self = Self::Playing;
        }
    }

    pub fn check_goal(&mut self, player: &Player, goal: Goal) {
        if *self != Self::Playing {
            return;
        }

        let distance_squared = (player.x - goal.x).powi(2) + (player.y - goal.y).powi(2);
        if distance_squared <= GOAL_RADIUS * GOAL_RADIUS {
            *self = Self::Success;
        }
    }

    pub fn return_to_menu(&mut self) {
        if *self == Self::Success {
            *self = Self::Menu;
        }
    }
}

pub fn draw_menu(buffer: &mut [u32], width: usize, height: usize) {
    buffer.fill(MENU_BACKGROUND);
    bitmap_font::draw_centered(
        buffer,
        width,
        height,
        height / 3,
        b"PROYECTO 1 - RAY CASTER",
        3,
        TITLE_COLOR,
    );
    bitmap_font::draw_centered(
        buffer,
        width,
        height,
        height / 2,
        b"ENTER O ESPACIO PARA JUGAR",
        2,
        TEXT_COLOR,
    );
    bitmap_font::draw_centered(
        buffer,
        width,
        height,
        height * 2 / 3,
        b"WASD Y MOUSE PARA MOVERTE",
        2,
        ACCENT_COLOR,
    );
    bitmap_font::draw_centered(
        buffer,
        width,
        height,
        height * 2 / 3 + 42,
        b"BUSCA LA META VERDE EN EL MINIMAPA",
        2,
        ACCENT_COLOR,
    );
}

pub fn draw_success(buffer: &mut [u32], width: usize, height: usize) {
    buffer.fill(SUCCESS_BACKGROUND);
    bitmap_font::draw_centered(buffer, width, height, height / 3, b"EXITO", 5, TITLE_COLOR);
    bitmap_font::draw_centered(
        buffer,
        width,
        height,
        height / 2,
        b"LLEGASTE A LA META",
        2,
        TEXT_COLOR,
    );
    bitmap_font::draw_centered(
        buffer,
        width,
        height,
        height * 2 / 3,
        b"ENTER PARA VOLVER AL MENU",
        2,
        ACCENT_COLOR,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::level::PlayerStart;

    #[test]
    fn transiciona_de_menu_a_juego() {
        let mut state = GameState::Menu;
        state.start();
        assert_eq!(state, GameState::Playing);
    }

    #[test]
    fn llega_al_exito_al_entrar_en_la_meta() {
        let mut state = GameState::Playing;
        let player = Player::from(PlayerStart {
            x: 2.5,
            y: 3.5,
            angle: 0.0,
        });
        state.check_goal(&player, Goal { x: 2.5, y: 3.5 });
        assert_eq!(state, GameState::Success);
    }
}
