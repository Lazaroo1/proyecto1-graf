use minifb::{Key, Window, WindowOptions};

mod level;
mod raycaster;

const WIDTH: usize = 800;
const HEIGHT: usize = 600;

fn main() {
    let level = level::Level::parse(include_str!("../assets/niveles/prueba.txt"))
        .expect("No se pudo cargar el nivel de prueba");
    let player = level.player_start;

    let mut window = Window::new(
        "Proyecto 1 - Ray Caster",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .expect("No se pudo crear la ventana");

    window.set_target_fps(60);

    let mut buffer = vec![0_u32; WIDTH * HEIGHT];

    while window.is_open() && !window.is_key_down(Key::Escape) {
        raycaster::render(
            &mut buffer,
            WIDTH,
            HEIGHT,
            &level,
            player,
            raycaster::DEFAULT_FOV,
        );
        window
            .update_with_buffer(&buffer, WIDTH, HEIGHT)
            .expect("No se pudo actualizar la ventana");
    }
}
