pub(super) mod repeat;

use smithay_client_toolkit::seat::keyboard::{
    KeyEvent, KeyboardHandler, Keymap, Keysym, Modifiers, RawModifiers, RepeatInfo,
};
use wayland_client::{
    Connection, QueueHandle,
    protocol::{wl_keyboard::WlKeyboard, wl_surface::WlSurface},
};

use crate::Key;
use crate::input::focus;

use super::WaylandState;

impl KeyboardHandler for WaylandState {
    // keys only say which keyboard they came from, so the focused window is kept here
    fn enter(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &WlKeyboard,
        surface: &WlSurface,
        _: u32,
        _: &[u32],
        _: &[Keysym],
    ) {
        self.keyboard_focus = Some(surface.clone());
    }

    fn leave(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &WlKeyboard,
        surface: &WlSurface,
        _: u32,
    ) {
        self.keyboard_focus = None;
        self.keyboard_repeat.cancel(&self.loop_handle);

        // keys stop arriving, so no text input can stay focused
        focus::clear();

        self.request_frame_on(surface);
    }

    fn press_key(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &WlKeyboard,
        _: u32,
        event: KeyEvent,
    ) {
        let Some(surface) = self.keyboard_focus.clone() else {
            return;
        };

        let key = to_key(&event);

        // a focused text input takes the key before the window's on_key sees it
        let taken = focus::send(key, event.raw_code);
        self.keyboard_repeat.press(
            event,
            taken && focus::repeats(key),
            &self.loop_handle,
            Self::repeat_text_input,
        );

        if taken {
            self.request_frame_on(&surface);

            return;
        }

        let Some(window) = self.window(&surface) else {
            return;
        };

        let Some(on_key) = &window.on_key else {
            return;
        };

        on_key(key);

        // a service the handler changed wakes its own readers, this window redraws for the rest
        self.request_frame_on(&surface);
    }

    fn repeat_key(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        keyboard: &WlKeyboard,
        _: u32,
        event: KeyEvent,
    ) {
        if self.keyboard_device.as_ref() == Some(keyboard) {
            let event = self.keyboard_repeat.native(event);
            self.repeat_text_input(event);
        }
    }

    fn release_key(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &WlKeyboard,
        _: u32,
        event: KeyEvent,
    ) {
        focus::release(event.raw_code);
        self.keyboard_repeat
            .release(event.raw_code, &self.loop_handle);
    }

    fn update_modifiers(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &WlKeyboard,
        _: u32,
        _: Modifiers,
        modifiers: RawModifiers,
        group: u32,
    ) {
        self.keyboard_repeat.modifiers(modifiers, group);
    }

    fn update_repeat_info(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &WlKeyboard,
        info: RepeatInfo,
    ) {
        self.keyboard_repeat
            .configure(info, &self.loop_handle, Self::repeat_text_input);
    }

    fn update_keymap(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &WlKeyboard,
        keymap: Keymap<'_>,
    ) {
        self.keyboard_repeat.cancel(&self.loop_handle);
        self.keyboard_repeat.keymap(&keymap.as_string());
    }
}

impl WaylandState {
    pub fn repeat_text_input(&mut self, event: KeyEvent) -> bool {
        let Some(surface) = self.keyboard_focus.clone() else {
            return false;
        };

        if self.keyboard_device.is_none()
            || self.window(&surface).is_none()
            || !focus::has_repeat(event.raw_code)
        {
            return false;
        }

        if focus::repeat(to_key(&event), event.raw_code) {
            self.request_frame_on(&surface);
        }
        true
    }
}

fn to_key(event: &KeyEvent) -> Key {
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
        Keysym::Home => Key::Home,
        Keysym::End => Key::End,
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
