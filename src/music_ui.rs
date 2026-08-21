use crate::bitmap_font;
use crate::music::{MusicPlayer, MusicSnapshot};
use minifb::{MouseButton, MouseMode, Window};
use std::time::Duration;

const PANEL_WIDTH: usize = 252;
const PANEL_HEIGHT: usize = 126;
const MARGIN: usize = 10;
const PANEL_COLOR: u32 = 0x24272D;
const BORDER_COLOR: u32 = 0xB94E48;
const INNER_BORDER_COLOR: u32 = 0x552A2A;
const TEXT_COLOR: u32 = 0xF0D59A;
const SECONDARY_TEXT_COLOR: u32 = 0xE8E9ED;
const TRACK_COLOR: u32 = 0x111319;
const PROGRESS_COLOR: u32 = 0xC44F49;
const BUTTON_COLOR: u32 = 0x743633;
const BUTTON_HOVER_COLOR: u32 = 0x9B4641;
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

        bitmap_font::draw_text(
            buffer,
            width,
            height,
            panel.x + 12,
            panel.y + 12,
            snapshot.track_name.as_bytes(),
            2,
            if snapshot.available {
                TEXT_COLOR
            } else {
                MUTED_COLOR
            },
        );

        let progress_track = Rect {
            x: panel.x + 12,
            y: panel.y + 37,
            width: panel.width - 24,
            height: 10,
        };
        fill_rect(buffer, width, height, progress_track, TRACK_COLOR);
        stroke_rect(buffer, width, height, progress_track, INNER_BORDER_COLOR);
        let ratio = progress_ratio(snapshot.elapsed, snapshot.duration);
        let progress_width = ((progress_track.width - 2) as f32 * ratio).round() as usize;
        fill_rect(
            buffer,
            width,
            height,
            Rect {
                x: progress_track.x + 1,
                y: progress_track.y + 1,
                width: progress_width,
                height: progress_track.height - 2,
            },
            PROGRESS_COLOR,
        );

        let time = format_times(snapshot.elapsed, snapshot.duration);
        let time_width = time.len() * (bitmap_font::GLYPH_WIDTH + 1) - 1;
        bitmap_font::draw_text(
            buffer,
            width,
            height,
            panel.x + (panel.width - time_width) / 2,
            panel.y + 54,
            &time,
            1,
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
            panel.x + 88,
            panel.y + 112,
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
    let y = panel.y + 74;
    (
        Rect {
            x: panel.x + 52,
            y,
            width: 34,
            height: 30,
        },
        Rect {
            x: panel.x + 109,
            y,
            width: 34,
            height: 30,
        },
        Rect {
            x: panel.x + 166,
            y,
            width: 34,
            height: 30,
        },
    )
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
        rect,
        if hovered {
            BUTTON_HOVER_COLOR
        } else {
            BUTTON_COLOR
        },
    );
    stroke_rect(buffer, width, height, rect, BORDER_COLOR);
}

fn draw_play_pause_icon(buffer: &mut [u32], width: usize, height: usize, rect: Rect, paused: bool) {
    if paused {
        for row in 0..14 {
            for column in 0..=(row.min(13 - row) / 2) {
                put_pixel(
                    buffer,
                    width,
                    height,
                    rect.x + 12 + column,
                    rect.y + 8 + row,
                    TEXT_COLOR,
                );
            }
        }
    } else {
        fill_rect(
            buffer,
            width,
            height,
            Rect {
                x: rect.x + 10,
                y: rect.y + 8,
                width: 4,
                height: 14,
            },
            TEXT_COLOR,
        );
        fill_rect(
            buffer,
            width,
            height,
            Rect {
                x: rect.x + 20,
                y: rect.y + 8,
                width: 4,
                height: 14,
            },
            TEXT_COLOR,
        );
    }
}

fn draw_next_icon(buffer: &mut [u32], width: usize, height: usize, rect: Rect) {
    for row in 0..14 {
        let span = row.min(13 - row) / 2;
        for column in 0..=span {
            put_pixel(
                buffer,
                width,
                height,
                rect.x + 10 + column,
                rect.y + 8 + row,
                TEXT_COLOR,
            );
        }
    }
    fill_rect(
        buffer,
        width,
        height,
        Rect {
            x: rect.x + 21,
            y: rect.y + 8,
            width: 3,
            height: 14,
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
            y: rect.y + 12,
            width: 5,
            height: 7,
        },
        TEXT_COLOR,
    );
    for offset in 0..6 {
        put_pixel(
            buffer,
            width,
            height,
            rect.x + 13 + offset / 2,
            rect.y + 11 - offset / 2,
            TEXT_COLOR,
        );
        put_pixel(
            buffer,
            width,
            height,
            rect.x + 13 + offset / 2,
            rect.y + 19 + offset / 2,
            TEXT_COLOR,
        );
    }
    if muted {
        for offset in 0..15 {
            put_pixel(
                buffer,
                width,
                height,
                rect.x + 9 + offset,
                rect.y + 7 + offset,
                MUTED_COLOR,
            );
        }
    } else {
        for offset in 0..8 {
            put_pixel(
                buffer,
                width,
                height,
                rect.x + 20 + offset / 3,
                rect.y + 11 + offset,
                TEXT_COLOR,
            );
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
