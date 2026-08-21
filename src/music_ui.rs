use crate::bitmap_font;
use crate::music::{MusicPlayer, MusicSnapshot};
use minifb::{MouseButton, MouseMode, Window};
use std::time::Duration;

const PANEL_WIDTH: usize = 228;
const PANEL_HEIGHT: usize = 174;
const MARGIN: usize = 10;
const SHADOW_COLOR: u32 = 0x090A0D;
const PANEL_COLOR: u32 = 0x303238;
const PANEL_INNER_COLOR: u32 = 0x292B30;
const BORDER_COLOR: u32 = 0xB94E48;
const BORDER_HIGHLIGHT_COLOR: u32 = 0xE06A60;
const INNER_BORDER_COLOR: u32 = 0x562826;
const TEXT_COLOR: u32 = 0xF3C982;
const SECONDARY_TEXT_COLOR: u32 = 0xE8E9ED;
const TRACK_COLOR: u32 = 0x131419;
const PROGRESS_COLOR: u32 = 0xC44F49;
const PROGRESS_HIGHLIGHT_COLOR: u32 = 0xE26A60;
const BUTTON_COLOR: u32 = 0x703330;
const BUTTON_INNER_COLOR: u32 = 0x8D403B;
const BUTTON_HOVER_COLOR: u32 = 0xA94C46;
const MUTED_COLOR: u32 = 0xD66A61;

#[derive(Clone, Copy)]
struct Rect {
    x: usize,
    y: usize,
    width: usize,
    height: usize,
}

impl Rect {
    fn contains(self, x: usize, y: usize) -> bool {
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }
}

pub struct MusicUi {
    mouse_was_down: bool,
    mouse_position: Option<(usize, usize)>,
}

impl MusicUi {
    pub fn new() -> Self {
        Self {
            mouse_was_down: false,
            mouse_position: None,
        }
    }

    pub fn handle_mouse(
        &mut self,
        window: &Window,
        music: &MusicPlayer,
        controls_enabled: bool,
        screen_width: usize,
        screen_height: usize,
    ) {
        let mouse_down = window.get_mouse_down(MouseButton::Left);
        let clicked = mouse_down && !self.mouse_was_down;
        self.mouse_was_down = mouse_down;
        self.mouse_position = window
            .get_mouse_pos(MouseMode::Discard)
            .map(|(x, y)| (x.max(0.0) as usize, y.max(0.0) as usize));

        if !clicked || !controls_enabled {
            return;
        }

        let Some((mouse_x, mouse_y)) = self.mouse_position else {
            return;
        };
        let (play_button, next_button, mute_button) = button_rects(screen_width, screen_height);

        if play_button.contains(mouse_x, mouse_y) {
            music.toggle_pause();
        } else if next_button.contains(mouse_x, mouse_y) {
            music.next();
        } else if mute_button.contains(mouse_x, mouse_y) {
            music.toggle_mute();
        }
    }

    pub fn draw(
        &self,
        buffer: &mut [u32],
        width: usize,
        height: usize,
        snapshot: MusicSnapshot,
        controls_enabled: bool,
    ) {
        if buffer.len() != width.saturating_mul(height) || width == 0 || height == 0 {
            return;
        }

        let panel = panel_rect(width, height);
        fill_rect(
            buffer,
            width,
            height,
            Rect {
                x: panel.x + 5,
                y: panel.y + 5,
                width: panel.width,
                height: panel.height,
            },
            SHADOW_COLOR,
        );
        fill_rect(buffer, width, height, panel, PANEL_COLOR);
        stroke_rect(buffer, width, height, panel, BORDER_COLOR);
        stroke_rect(
            buffer,
            width,
            height,
            Rect {
                x: panel.x + 3,
                y: panel.y + 3,
                width: panel.width - 6,
                height: panel.height - 6,
            },
            INNER_BORDER_COLOR,
        );
        fill_rect(
            buffer,
            width,
            height,
            Rect {
                x: panel.x + 7,
                y: panel.y + 7,
                width: panel.width - 14,
                height: panel.height - 14,
            },
            PANEL_INNER_COLOR,
        );
        draw_corner_details(buffer, width, height, panel);

        draw_panel_centered_text(
            buffer,
            width,
            height,
            panel,
            panel.y + 18,
            snapshot.track_name.as_bytes(),
            if snapshot.track_name.len() <= 17 {
                2
            } else {
                1
            },
            if snapshot.available {
                TEXT_COLOR
            } else {
                MUTED_COLOR
            },
        );

        let progress_track = Rect {
            x: panel.x + 17,
            y: panel.y + 51,
            width: panel.width - 34,
            height: 20,
        };
        fill_rect(buffer, width, height, progress_track, TRACK_COLOR);
        stroke_rect(buffer, width, height, progress_track, SHADOW_COLOR);
        stroke_rect(
            buffer,
            width,
            height,
            Rect {
                x: progress_track.x + 1,
                y: progress_track.y + 1,
                width: progress_track.width - 2,
                height: progress_track.height - 2,
            },
            INNER_BORDER_COLOR,
        );
        let ratio = progress_ratio(snapshot.elapsed, snapshot.duration);
        let progress_width = ((progress_track.width - 6) as f32 * ratio).round() as usize;
        fill_rect(
            buffer,
            width,
            height,
            Rect {
                x: progress_track.x + 3,
                y: progress_track.y + 3,
                width: progress_width,
                height: progress_track.height - 6,
            },
            PROGRESS_COLOR,
        );
        if progress_width > 0 {
            fill_rect(
                buffer,
                width,
                height,
                Rect {
                    x: progress_track.x + 3,
                    y: progress_track.y + 3,
                    width: progress_width,
                    height: 2,
                },
                PROGRESS_HIGHLIGHT_COLOR,
            );
        }

        let time = format_times(snapshot.elapsed, snapshot.duration);
        draw_panel_centered_text(
            buffer,
            width,
            height,
            panel,
            panel.y + 82,
            &time,
            2,
            SECONDARY_TEXT_COLOR,
        );

        let mouse = self.mouse_position.filter(|_| controls_enabled);
        let (play_button, next_button, mute_button) = button_rects(width, height);
        draw_button(
            buffer,
            width,
            height,
            play_button,
            mouse.is_some_and(|point| play_button.contains(point.0, point.1)),
        );
        draw_button(
            buffer,
            width,
            height,
            next_button,
            mouse.is_some_and(|point| next_button.contains(point.0, point.1)),
        );
        draw_button(
            buffer,
            width,
            height,
            mute_button,
            mouse.is_some_and(|point| mute_button.contains(point.0, point.1)),
        );
        draw_play_pause_icon(buffer, width, height, play_button, snapshot.paused);
        draw_next_icon(buffer, width, height, next_button);
        draw_mute_icon(buffer, width, height, mute_button, snapshot.muted);

        bitmap_font::draw_text(
            buffer,
            width,
            height,
            panel.x + 82,
            panel.y + 158,
            if controls_enabled {
                b"TAB VOLVER"
            } else {
                b"TAB MOUSE"
            },
            1,
            SECONDARY_TEXT_COLOR,
        );
    }
}

fn panel_rect(width: usize, height: usize) -> Rect {
    Rect {
        x: width.saturating_sub(PANEL_WIDTH + MARGIN),
        y: height.saturating_sub(PANEL_HEIGHT + MARGIN),
        width: PANEL_WIDTH.min(width),
        height: PANEL_HEIGHT.min(height),
    }
}

fn button_rects(width: usize, height: usize) -> (Rect, Rect, Rect) {
    let panel = panel_rect(width, height);
    let y = panel.y + 112;
    (
        Rect {
            x: panel.x + 34,
            y,
            width: 40,
            height: 38,
        },
        Rect {
            x: panel.x + 94,
            y,
            width: 40,
            height: 38,
        },
        Rect {
            x: panel.x + 154,
            y,
            width: 40,
            height: 38,
        },
    )
}

#[allow(clippy::too_many_arguments)]
fn draw_panel_centered_text(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    panel: Rect,
    y: usize,
    text: &[u8],
    scale: usize,
    color: u32,
) {
    let text_width = text
        .len()
        .saturating_mul((bitmap_font::GLYPH_WIDTH + 1) * scale)
        .saturating_sub(scale);
    let x = panel.x + panel.width.saturating_sub(text_width) / 2;
    bitmap_font::draw_text(buffer, width, height, x, y, text, scale, color);
}

fn draw_corner_details(buffer: &mut [u32], width: usize, height: usize, panel: Rect) {
    let corners = [
        (panel.x + 5, panel.y + 5, 1_i32, 1_i32),
        (panel.x + panel.width - 6, panel.y + 5, -1, 1),
        (panel.x + 5, panel.y + panel.height - 6, 1, -1),
        (
            panel.x + panel.width - 6,
            panel.y + panel.height - 6,
            -1,
            -1,
        ),
    ];

    for (x, y, direction_x, direction_y) in corners {
        for offset in 0..7_i32 {
            put_pixel_signed(
                buffer,
                width,
                height,
                x as i32 + offset * direction_x,
                y as i32,
                BORDER_HIGHLIGHT_COLOR,
            );
            put_pixel_signed(
                buffer,
                width,
                height,
                x as i32,
                y as i32 + offset * direction_y,
                BORDER_HIGHLIGHT_COLOR,
            );
        }
        fill_rect(
            buffer,
            width,
            height,
            Rect {
                x: x.saturating_sub(1),
                y: y.saturating_sub(1),
                width: 3,
                height: 3,
            },
            TEXT_COLOR,
        );
    }
}

fn progress_ratio(elapsed: Duration, total: Duration) -> f32 {
    if total.is_zero() {
        0.0
    } else {
        (elapsed.as_secs_f32() / total.as_secs_f32()).clamp(0.0, 1.0)
    }
}

fn format_times(elapsed: Duration, total: Duration) -> [u8; 13] {
    let (elapsed_minutes, elapsed_seconds) = minutes_seconds(elapsed);
    let (total_minutes, total_seconds) = minutes_seconds(total);
    [
        b'0' + elapsed_minutes / 10,
        b'0' + elapsed_minutes % 10,
        b':',
        b'0' + elapsed_seconds / 10,
        b'0' + elapsed_seconds % 10,
        b' ',
        b'/',
        b' ',
        b'0' + total_minutes / 10,
        b'0' + total_minutes % 10,
        b':',
        b'0' + total_seconds / 10,
        b'0' + total_seconds % 10,
    ]
}

fn minutes_seconds(duration: Duration) -> (u8, u8) {
    let seconds = duration.as_secs().min(99 * 60 + 59);
    ((seconds / 60) as u8, (seconds % 60) as u8)
}

fn draw_button(buffer: &mut [u32], width: usize, height: usize, rect: Rect, hovered: bool) {
    fill_rect(
        buffer,
        width,
        height,
        Rect {
            x: rect.x + 3,
            y: rect.y + 3,
            width: rect.width,
            height: rect.height,
        },
        SHADOW_COLOR,
    );
    fill_rect(
        buffer,
        width,
        height,
        rect,
        if hovered {
            BUTTON_HOVER_COLOR
        } else {
            BUTTON_COLOR
        },
    );
    stroke_rect(buffer, width, height, rect, BORDER_COLOR);
    stroke_rect(
        buffer,
        width,
        height,
        Rect {
            x: rect.x + 3,
            y: rect.y + 3,
            width: rect.width - 6,
            height: rect.height - 6,
        },
        BUTTON_INNER_COLOR,
    );
}

fn draw_play_pause_icon(buffer: &mut [u32], width: usize, height: usize, rect: Rect, paused: bool) {
    if paused {
        draw_right_triangle(buffer, width, height, rect.x + 12, rect.y + 10, 11, 18);
    } else {
        fill_rect(
            buffer,
            width,
            height,
            Rect {
                x: rect.x + 11,
                y: rect.y + 10,
                width: 5,
                height: 18,
            },
            TEXT_COLOR,
        );
        fill_rect(
            buffer,
            width,
            height,
            Rect {
                x: rect.x + 24,
                y: rect.y + 10,
                width: 5,
                height: 18,
            },
            TEXT_COLOR,
        );
    }
}

fn draw_next_icon(buffer: &mut [u32], width: usize, height: usize, rect: Rect) {
    draw_right_triangle(buffer, width, height, rect.x + 9, rect.y + 10, 11, 18);
    fill_rect(
        buffer,
        width,
        height,
        Rect {
            x: rect.x + 27,
            y: rect.y + 10,
            width: 4,
            height: 18,
        },
        TEXT_COLOR,
    );
}

fn draw_mute_icon(buffer: &mut [u32], width: usize, height: usize, rect: Rect, muted: bool) {
    fill_rect(
        buffer,
        width,
        height,
        Rect {
            x: rect.x + 8,
            y: rect.y + 15,
            width: 7,
            height: 9,
        },
        TEXT_COLOR,
    );
    for offset in 0..8 {
        put_pixel(
            buffer,
            width,
            height,
            rect.x + 15 + offset / 2,
            rect.y + 14 - offset / 2,
            TEXT_COLOR,
        );
        put_pixel(
            buffer,
            width,
            height,
            rect.x + 15 + offset / 2,
            rect.y + 24 + offset / 2,
            TEXT_COLOR,
        );
    }
    if muted {
        for offset in 0..21 {
            put_pixel(
                buffer,
                width,
                height,
                rect.x + 8 + offset,
                rect.y + 8 + offset,
                MUTED_COLOR,
            );
        }
    } else {
        for offset in 0..12 {
            put_pixel(
                buffer,
                width,
                height,
                rect.x + 24 + offset / 4,
                rect.y + 13 + offset,
                TEXT_COLOR,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_right_triangle(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    origin_x: usize,
    origin_y: usize,
    triangle_width: usize,
    triangle_height: usize,
) {
    if triangle_width < 2 || triangle_height < 2 {
        return;
    }
    let center_y = origin_y + triangle_height / 2;
    for column in 0..triangle_width {
        let half_span =
            (triangle_width - 1 - column) * (triangle_height / 2) / (triangle_width - 1);
        for y in center_y.saturating_sub(half_span)..=center_y + half_span {
            put_pixel(buffer, width, height, origin_x + column, y, TEXT_COLOR);
        }
    }
}

fn fill_rect(buffer: &mut [u32], width: usize, height: usize, rect: Rect, color: u32) {
    let max_y = (rect.y + rect.height).min(height);
    let max_x = (rect.x + rect.width).min(width);
    for y in rect.y.min(height)..max_y {
        buffer[y * width + rect.x.min(width)..y * width + max_x].fill(color);
    }
}

fn stroke_rect(buffer: &mut [u32], width: usize, height: usize, rect: Rect, color: u32) {
    if rect.width == 0 || rect.height == 0 {
        return;
    }
    for x in rect.x..rect.x + rect.width {
        put_pixel(buffer, width, height, x, rect.y, color);
        put_pixel(buffer, width, height, x, rect.y + rect.height - 1, color);
    }
    for y in rect.y..rect.y + rect.height {
        put_pixel(buffer, width, height, rect.x, y, color);
        put_pixel(buffer, width, height, rect.x + rect.width - 1, y, color);
    }
}

fn put_pixel(buffer: &mut [u32], width: usize, height: usize, x: usize, y: usize, color: u32) {
    if x < width && y < height {
        buffer[y * width + x] = color
    }
}

fn put_pixel_signed(buffer: &mut [u32], width: usize, height: usize, x: i32, y: i32, color: u32) {
    let (Ok(x), Ok(y)) = (usize::try_from(x), usize::try_from(y)) else {
        return;
    };
    put_pixel(buffer, width, height, x, y, color);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formatea_tiempos_reales_en_minutos_y_segundos() {
        assert_eq!(
            &format_times(Duration::from_secs(227), Duration::from_secs(261)),
            b"03:47 / 04:21"
        );
    }

    #[test]
    fn limita_el_progreso_al_intervalo_visible() {
        assert_eq!(
            progress_ratio(Duration::from_secs(20), Duration::from_secs(10)),
            1.0
        );
        assert_eq!(progress_ratio(Duration::ZERO, Duration::ZERO), 0.0);
    }

    #[test]
    fn dibuja_panel_progreso_y_controles_superpuestos() {
        let ui = MusicUi::new();
        let mut buffer = vec![0; 800 * 600];
        ui.draw(
            &mut buffer,
            800,
            600,
            MusicSnapshot {
                track_name: "STYLE",
                elapsed: Duration::from_secs(30),
                duration: Duration::from_secs(120),
                paused: false,
                muted: false,
                available: true,
            },
            false,
        );

        assert!(buffer.contains(&PANEL_COLOR));
        assert!(buffer.contains(&PROGRESS_COLOR));
        assert!(buffer.contains(&BUTTON_COLOR));
        assert!(buffer.contains(&TEXT_COLOR));
    }
}
