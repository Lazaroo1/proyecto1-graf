use minifb::{Key, KeyRepeat, Window, WindowOptions};
use std::time::Instant;

mod bitmap_font;
mod fps_counter;
mod game_state;
mod level;
mod minimap;
mod mouse_look;
mod music;
mod music_ui;
mod player;
mod raycaster;
mod texture;

const WIDTH: usize = 800;
const HEIGHT: usize = 600;
const MOVEMENT_SPEED: f32 = 3.0;
const ROTATION_SPEED: f32 = 2.0;
const MOUSE_SENSITIVITY: f32 = 0.0025;
const MAX_FRAME_TIME: f32 = 0.1;

fn main() {
    let level = level::Level::parse(include_str!("../assets/niveles/prueba.txt"))
        .expect("No se pudo cargar el nivel de prueba");
    let mut player = player::Player::from(level.player_start);
    let textures = texture::WallTextures::load_embedded()
        .expect("No se pudieron cargar las texturas de paredes");

    let mut window = Window::new(
        "Proyecto 1 - Ray Caster",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .expect("No se pudo crear la ventana");

    window.set_target_fps(60);

    let mut buffer = vec![0_u32; WIDTH * HEIGHT];
    let mut previous_frame = Instant::now();
    let mut mouse_look = mouse_look::MouseLook::new();
    let mut fps_counter = fps_counter::FpsCounter::new();
    let raycaster = raycaster::Raycaster::new(WIDTH, HEIGHT, raycaster::DEFAULT_FOV);
    let minimap = minimap::Minimap::new(&level);
    let mut game_state = game_state::GameState::Menu;
    let mut music = music::MusicPlayer::new();
    let mut music_ui = music_ui::MusicUi::new();
    let mut music_controls_active = false;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let now = Instant::now();
        let delta_time = (now - previous_frame).as_secs_f32().min(MAX_FRAME_TIME);
        previous_frame = now;

        let start_pressed = window.is_key_pressed(Key::Enter, KeyRepeat::No)
            || window.is_key_pressed(Key::Space, KeyRepeat::No);
        let toggle_music_controls = window.is_key_pressed(Key::Tab, KeyRepeat::No);

        match game_state {
            game_state::GameState::Menu => {
                music_controls_active = false;
                mouse_look.release(&mut window);
                game_state::draw_menu(&mut buffer, WIDTH, HEIGHT);
                if start_pressed {
                    player = player::Player::from(level.player_start);
                    game_state.start();
                }
            }
            game_state::GameState::Playing => {
                if toggle_music_controls {
                    music_controls_active = !music_controls_active;
                }
                let mouse_delta_x = if music_controls_active {
                    mouse_look.release(&mut window);
                    0.0
                } else {
                    mouse_look.update(&mut window)
                };
                if window.is_key_pressed(Key::P, KeyRepeat::No) {
                    music.toggle_pause();
                }
                if window.is_key_pressed(Key::N, KeyRepeat::No) {
                    music.next();
                }
                if window.is_key_pressed(Key::M, KeyRepeat::No) {
                    music.toggle_mute();
                }
                music_ui.handle_mouse(&window, &music, music_controls_active, WIDTH, HEIGHT);
                update_player(&window, &level, &mut player, delta_time, mouse_delta_x);
                raycaster.render(&mut buffer, &level, &textures, &player);
                minimap.draw(&mut buffer, WIDTH, HEIGHT, &player);
                music_ui.draw(
                    &mut buffer,
                    WIDTH,
                    HEIGHT,
                    music.snapshot(),
                    music_controls_active,
                );
                game_state.check_goal(&player, level.goal);
            }
            game_state::GameState::Success => {
                music_controls_active = false;
                mouse_look.release(&mut window);
                game_state::draw_success(&mut buffer, WIDTH, HEIGHT);
                if start_pressed {
                    game_state.return_to_menu();
                }
            }
        }
        music.sync_game_state(game_state == game_state::GameState::Playing);
        fps_counter.draw(&mut buffer, WIDTH, HEIGHT);
        window
            .update_with_buffer(&buffer, WIDTH, HEIGHT)
            .expect("No se pudo actualizar la ventana");
        fps_counter.frame_rendered();
    }

    mouse_look.release(&mut window);
}

fn update_player(
    window: &Window,
    level: &level::Level,
    player: &mut player::Player,
    delta_time: f32,
    mouse_delta_x: f32,
) {
    let turn = key_axis(window, Key::Right, Key::Left);
    let rotation = turn * ROTATION_SPEED * delta_time + mouse_delta_x * MOUSE_SENSITIVITY;
    player.angle = (player.angle + rotation).rem_euclid(std::f32::consts::TAU);

    let mut forward = key_axis(window, Key::W, Key::S);
    let mut strafe = key_axis(window, Key::D, Key::A);
    let input_length = forward.hypot(strafe);

    if input_length > 1.0 {
        forward /= input_length;
        strafe /= input_length;
    }

    let direction_x = player.angle.cos();
    let direction_y = player.angle.sin();
    let distance = MOVEMENT_SPEED * delta_time;
    let delta_x = (direction_x * forward - direction_y * strafe) * distance;
    let delta_y = (direction_y * forward + direction_x * strafe) * distance;

    player.move_by(level, delta_x, delta_y);
}

fn key_axis(window: &Window, positive: Key, negative: Key) -> f32 {
    i32::from(window.is_key_down(positive)) as f32 - i32::from(window.is_key_down(negative)) as f32
}
