#[cfg(target_os = "windows")]
mod platform {
    use minifb::Window;
    use std::mem::zeroed;
    use std::ptr::null;
    use winapi::shared::windef::{HWND, POINT, RECT};
    use winapi::um::winuser::{
        ClientToScreen, ClipCursor, GetClientRect, GetCursorPos, ReleaseCapture, SetCapture,
        SetCursorPos,
    };

    pub struct MouseLook {
        captured: bool,
        center: Option<(i32, i32)>,
    }

    impl MouseLook {
        pub fn new() -> Self {
            Self {
                captured: false,
                center: None,
            }
        }

        pub fn update(&mut self, window: &mut Window) -> f32 {
            if !window.is_active() {
                self.release(window);
                return 0.0;
            }

            let handle = window.get_window_handle() as HWND;
            let Some((bounds, center)) = client_area(handle) else {
                self.release(window);
                return 0.0;
            };

            if !self.captured {
                // SAFETY: el handle pertenece a la ventana viva de minifb y las
                // estructuras se inicializan con coordenadas válidas de su cliente.
                let captured = unsafe {
                    SetCapture(handle);
                    let clipped = ClipCursor(&bounds) != 0;
                    let centered = SetCursorPos(center.0, center.1) != 0;
                    if !clipped || !centered {
                        ClipCursor(null());
                        ReleaseCapture();
                    }
                    clipped && centered
                };

                if captured {
                    window.set_cursor_visibility(false);
                    self.captured = true;
                    self.center = Some(center);
                }
                return 0.0;
            }

            // Si la ventana cambió de posición o tamaño, recentramos sin usar el
            // desplazamiento del centro anterior para evitar un salto de cámara.
            if self.center != Some(center) {
                unsafe {
                    ClipCursor(&bounds);
                    SetCursorPos(center.0, center.1);
                }
                self.center = Some(center);
                return 0.0;
            }

            let mut cursor: POINT = unsafe { zeroed() };
            unsafe {
                ClipCursor(&bounds);
                let read_cursor = GetCursorPos(&mut cursor) != 0;
                SetCursorPos(center.0, center.1);
                if read_cursor {
                    (cursor.x - center.0) as f32
                } else {
                    0.0
                }
            }
        }

        pub fn release(&mut self, window: &mut Window) {
            if self.captured {
                unsafe {
                    ClipCursor(null());
                    ReleaseCapture();
                }
                window.set_cursor_visibility(true);
                self.captured = false;
                self.center = None;
            }
        }
    }

    impl Drop for MouseLook {
        fn drop(&mut self) {
            if self.captured {
                unsafe {
                    ClipCursor(null());
                    ReleaseCapture();
                }
            }
        }
    }

    fn client_area(handle: HWND) -> Option<(RECT, (i32, i32))> {
        if handle.is_null() {
            return None;
        }

        let mut client: RECT = unsafe { zeroed() };
        if unsafe { GetClientRect(handle, &mut client) } == 0 {
            return None;
        }

        let mut top_left = POINT {
            x: client.left,
            y: client.top,
        };
        let mut bottom_right = POINT {
            x: client.right,
            y: client.bottom,
        };
        if unsafe { ClientToScreen(handle, &mut top_left) } == 0
            || unsafe { ClientToScreen(handle, &mut bottom_right) } == 0
        {
            return None;
        }

        let bounds = RECT {
            left: top_left.x,
            top: top_left.y,
            right: bottom_right.x,
            bottom: bottom_right.y,
        };
        if bounds.right <= bounds.left || bounds.bottom <= bounds.top {
            return None;
        }

        let center = (
            bounds.left + (bounds.right - bounds.left) / 2,
            bounds.top + (bounds.bottom - bounds.top) / 2,
        );
        Some((bounds, center))
    }
}

#[cfg(not(target_os = "windows"))]
mod platform {
    use minifb::{MouseMode, Window};

    pub struct MouseLook {
        captured: bool,
        previous_x: Option<f32>,
    }

    impl MouseLook {
        pub fn new() -> Self {
            Self {
                captured: false,
                previous_x: None,
            }
        }

        pub fn update(&mut self, window: &mut Window) -> f32 {
            if !window.is_active() {
                self.release(window);
                return 0.0;
            }

            let current_x = window
                .get_mouse_pos(MouseMode::Clamp)
                .map(|position| position.0);
            if !self.captured {
                window.set_cursor_visibility(false);
                self.captured = true;
                self.previous_x = current_x;
                return 0.0;
            }

            let delta = current_x
                .zip(self.previous_x)
                .map_or(0.0, |(current, previous)| current - previous);
            self.previous_x = current_x;
            delta
        }

        pub fn release(&mut self, window: &mut Window) {
            if self.captured {
                window.set_cursor_visibility(true);
                self.captured = false;
                self.previous_x = None;
            }
        }
    }
}

pub use platform::MouseLook;
