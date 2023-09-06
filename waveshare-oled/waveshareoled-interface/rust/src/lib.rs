//! waveshareoled waveshareoled Interface

mod waveshareoled;
use std::fmt::Display;

pub use waveshareoled::*;

/// Helper because our smithy doesn't support enums
#[derive(Debug, Clone)]
pub enum WrappedEvent {
    Button1Press,
    Button2Press,
    Button3Press,
    JoystickUp,
    JoystickDown,
    JoystickLeft,
    JoystickRight,
    JoystickPressed,
}

impl TryFrom<&str> for WrappedEvent {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "button1" => Ok(WrappedEvent::Button1Press),
            "button2" => Ok(WrappedEvent::Button2Press),
            "button3" => Ok(WrappedEvent::Button3Press),
            "joystick_up" => Ok(WrappedEvent::JoystickUp),
            "joystick_down" => Ok(WrappedEvent::JoystickDown),
            "joystick_left" => Ok(WrappedEvent::JoystickLeft),
            "joystick_right" => Ok(WrappedEvent::JoystickRight),
            "joystick_pressed" => Ok(WrappedEvent::JoystickPressed),
            _ => Err(anyhow::anyhow!("Invalid event")),
        }
    }
}

impl Display for WrappedEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            WrappedEvent::Button1Press => "button1",
            WrappedEvent::Button2Press => "button2",
            WrappedEvent::Button3Press => "button3",
            WrappedEvent::JoystickUp => "joystick_up",
            WrappedEvent::JoystickDown => "joystick_down",
            WrappedEvent::JoystickLeft => "joystick_left",
            WrappedEvent::JoystickRight => "joystick_right",
            WrappedEvent::JoystickPressed => "joystick_pressed",
        };
        write!(f, "{}", s)
    }
}
