use smithay_client_toolkit::seat::keyboard::{KeyEvent, Keysym};

use crate::Key;

pub fn translate(event: &KeyEvent) -> Key {
    match event.keysym {
        Keysym::Return | Keysym::KP_Enter => Key::Enter,
        Keysym::Escape => Key::Escape,
        Keysym::Tab => Key::Tab,
        Keysym::BackSpace => Key::Backspace,
        Keysym::space => Key::Space,
        Keysym::Up => Key::Up,
        Keysym::Down => Key::Down,
        Keysym::Left => Key::Left,
        Keysym::Right => Key::Right,
        _ => character(event),
    }
}

// the text a key types already has shift and the keyboard layout applied
fn character(event: &KeyEvent) -> Key {
    let Some(text) = &event.utf8 else {
        return Key::Other;
    };

    let mut letters = text.chars();

    // a key that types more than one character has no single letter to give
    let (Some(letter), None) = (letters.next(), letters.next()) else {
        return Key::Other;
    };

    // with ctrl held, keys type invisible control characters
    if letter.is_control() {
        return Key::Other;
    }

    Key::Character(letter)
}
