use gilrs::{Axis, Button, EventType, Gilrs};

const LEFT_STICK_DEADZONE: f32 = 0.2;
const RIGHT_STICK_DEADZONE: f32 = 0.18;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct GamepadFrame {
    pub forward: f32,
    pub strafe: f32,
    pub turn: f32,
    pub start_pressed: bool,
}

pub struct GamepadInput {
    gilrs: Option<Gilrs>,
}

impl GamepadInput {
    pub fn new() -> Self {
        Self {
            // La ausencia de control o un backend no disponible no impiden
            // iniciar el juego con teclado y mouse.
            gilrs: Gilrs::new().ok(),
        }
    }

    pub fn poll(&mut self, accept_input: bool) -> GamepadFrame {
        let Some(gilrs) = self.gilrs.as_mut() else {
            return GamepadFrame::default();
        };

        let mut start_pressed = false;
        while let Some(event) = gilrs.next_event() {
            if accept_input
                && matches!(
                    event.event,
                    EventType::ButtonPressed(Button::South | Button::Start, _)
                )
            {
                start_pressed = true;
            }
        }

        if !accept_input {
            return GamepadFrame::default();
        }

        let Some((_id, gamepad)) = gilrs.gamepads().next() else {
            return GamepadFrame {
                start_pressed,
                ..GamepadFrame::default()
            };
        };

        let (strafe, forward) = radial_deadzone(
            gamepad.value(Axis::LeftStickX),
            gamepad.value(Axis::LeftStickY),
            LEFT_STICK_DEADZONE,
        );
        let turn = axial_deadzone(gamepad.value(Axis::RightStickX), RIGHT_STICK_DEADZONE);

        GamepadFrame {
            forward,
            strafe,
            turn,
            start_pressed,
        }
    }
}

fn axial_deadzone(value: f32, deadzone: f32) -> f32 {
    let magnitude = value.abs();
    if !value.is_finite() || magnitude <= deadzone {
        return 0.0;
    }
    value.signum() * ((magnitude.min(1.0) - deadzone) / (1.0 - deadzone))
}

fn radial_deadzone(x: f32, y: f32, deadzone: f32) -> (f32, f32) {
    if !x.is_finite() || !y.is_finite() {
        return (0.0, 0.0);
    }
    let magnitude = x.hypot(y);
    if magnitude <= deadzone {
        return (0.0, 0.0);
    }

    let scaled_magnitude = (magnitude.min(1.0) - deadzone) / (1.0 - deadzone);
    let scale = scaled_magnitude / magnitude;
    (x * scale, y * scale)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn elimina_drift_dentro_de_las_zonas_muertas() {
        assert_eq!(radial_deadzone(0.12, -0.1, LEFT_STICK_DEADZONE), (0.0, 0.0));
        assert_eq!(axial_deadzone(0.15, RIGHT_STICK_DEADZONE), 0.0);
        assert_eq!(axial_deadzone(f32::NAN, RIGHT_STICK_DEADZONE), 0.0);
    }

    #[test]
    fn reescala_sticks_fuera_de_la_zona_muerta() {
        assert!((axial_deadzone(1.0, RIGHT_STICK_DEADZONE) - 1.0).abs() < f32::EPSILON);
        let (x, y) = radial_deadzone(0.8, 0.8, LEFT_STICK_DEADZONE);
        assert!(x.hypot(y) <= 1.0);
        assert!(x > 0.0 && y > 0.0);
    }
}
