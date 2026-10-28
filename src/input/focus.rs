use std::cell::{Cell, RefCell};

use crate::Key;

use super::KeyHandler;

// the text input that typed keys go to, and what it does with them
struct Focus {
    window: u32,
    id: &'static str,
    on_key: KeyHandler,
}

thread_local! {
    // only one text input takes keys at a time
    static FOCUS: RefCell<Option<Focus>> = const { RefCell::new(None) };

    // the window being drawn right now, so a text input knows which window it is in
    static DRAWING: Cell<u32> = const { Cell::new(0) };
}

// called before drawing a window's widgets
pub fn start_drawing(window: u32) {
    DRAWING.set(window);
}

pub fn drawing() -> u32 {
    DRAWING.get()
}

pub fn set(window: u32, id: &'static str, on_key: KeyHandler) {
    FOCUS.set(Some(Focus { window, id, on_key }));
}

pub fn clear() {
    FOCUS.set(None);
}

pub fn has(id: &'static str) -> bool {
    FOCUS.with_borrow(|focus| matches!(focus, Some(focus) if focus.id == id))
}

// returns whether a text input took the key
pub fn send(window: u32, key: Key) -> bool {
    // a text input has no use for these, so they go on to the window's on_key
    if matches!(key, Key::Up | Key::Down | Key::Tab | Key::Other) {
        return false;
    }

    // copied out first, so the handler is free to change the focus itself
    let on_key = FOCUS.with_borrow(|focus| {
        focus
            .as_ref()
            .filter(|focus| focus.window == window)
            .map(|focus| focus.on_key.clone())
    });

    let Some(on_key) = on_key else {
        return false;
    };

    on_key(key);

    // escape leaves the input and still reaches the window, which may want to close
    key != Key::Escape
}
