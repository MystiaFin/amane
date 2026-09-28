use smithay_client_toolkit::seat::pointer::{BTN_LEFT, BTN_MIDDLE, BTN_RIGHT};

use crate::Button;

pub fn translate(code: u32) -> Option<Button> {
    match code {
        BTN_LEFT => Some(Button::Left),
        BTN_RIGHT => Some(Button::Right),
        BTN_MIDDLE => Some(Button::Middle),
        _ => None,
    }
}
