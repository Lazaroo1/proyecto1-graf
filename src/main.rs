use minifb::{Key, Window, WindowOptions};
use std::time::Instant;

mod level;
mod player;
mod raycaster;
mod texture;

const WIDTH: usize = 800;
const HEIGHT: usize = 600;
const MOVEMENT_SPEED: f32 = 3.0;
const ROTATION_SPEED: f32 = 2.0;
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

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let now = Instant::now();
        let delta_time = (now - previous_frame).as_secs_f32().min(MAX_FRAME_TIME);
        previous_frame = now;

        update_player(&window, &level, &mut player, delta_time);
        raycaster::render(
            &mut buffer,
            WIDTH,
            HEIGHT,
            &level,
            &textures,
            &player,
            raycaster::DEFAULT_FOV,
        );
        window
            .update_with_buffer(&buffer, WIDTH, HEIGHT)
            .expect("No se pudo actualizar la ventana");
    }
}

fn update_player(
    window: &Window,
    level: &level::Level,
    player: &mut player::Player,
    delta_time: f32,
) {
    let turn = key_axis(window, Key::Right, Key::Left);
    player.angle =
        (player.angle + turn * ROTATION_SPEED * delta_time).rem_euclid(std::f32::consts::TAU);

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
