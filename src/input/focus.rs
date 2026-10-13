use std::cell::RefCell;

use crate::Key;

use super::KeyHandler;

// the text input that typed keys go to, and what it does with them
struct Focus {
    id: &'static str,
    on_key: KeyHandler,
    repeating: Option<u32>,
}

thread_local! {
    // only one text input takes keys at a time
    static FOCUS: RefCell<Option<Focus>> = const { RefCell::new(None) };
}

pub fn set(id: &'static str, on_key: KeyHandler) {
    FOCUS.with_borrow_mut(|focus| {
        let repeating = focus
            .as_ref()
            .filter(|focus| focus.id == id)
            .and_then(|focus| focus.repeating);

        *focus = Some(Focus {
            id,
            on_key,
            repeating,
        });
    });
}

pub fn clear() {
    FOCUS.set(None);
}

pub fn has(id: &'static str) -> bool {
    FOCUS.with_borrow(|focus| matches!(focus, Some(focus) if focus.id == id))
}

// returns whether a text input took the key
pub fn send(key: Key, raw_code: u32) -> bool {
    // a text input has no use for these, so they go on to the window's on_key
    if matches!(key, Key::Up | Key::Down | Key::Tab | Key::Other) {
        return false;
    }

    // copied out first, so the handler is free to change the focus itself
    let on_key = FOCUS.with_borrow_mut(|focus| {
        focus.as_mut().map(|focus| {
            focus.repeating = repeats(key).then_some(raw_code);
            focus.on_key.clone()
        })
    });

    let Some(on_key) = on_key else {
        return false;
    };

    on_key(key);

    // escape leaves the input and still reaches the window, which may want to close
    key != Key::Escape
}

pub fn repeat(key: Key, raw_code: u32) -> bool {
    if !repeats(key) {
        return false;
    }

    let on_key = FOCUS.with_borrow(|focus| {
        focus
            .as_ref()
            .filter(|focus| focus.repeating == Some(raw_code))
            .map(|focus| focus.on_key.clone())
    });

    let Some(on_key) = on_key else {
        return false;
    };

    on_key(key);
    true
}

pub fn release(raw_code: u32) {
    FOCUS.with_borrow_mut(|focus| {
        if let Some(focus) = focus
            && focus.repeating == Some(raw_code)
        {
            focus.repeating = None;
        }
    });
}

pub fn has_repeat(raw_code: u32) -> bool {
    FOCUS.with_borrow(|focus| {
        focus
            .as_ref()
            .is_some_and(|focus| focus.repeating == Some(raw_code))
    })
}

pub fn repeats(key: Key) -> bool {
    matches!(
        key,
        Key::Character(_) | Key::Space | Key::Backspace | Key::Left | Key::Right
    )
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;

    fn record(id: &'static str) -> Rc<RefCell<Vec<Key>>> {
        let keys = Rc::new(RefCell::new(Vec::new()));
        let received = keys.clone();
        set(id, Rc::new(move |key| received.borrow_mut().push(key)));
        keys
    }

    #[test]
    fn repeats_text_editing_keys() {
        for key in [
            Key::Character('a'),
            Key::Space,
            Key::Backspace,
            Key::Left,
            Key::Right,
        ] {
            clear();
            let keys = record("editing");
            send(key, 30);
            assert!(repeat(key, 30));
            assert_eq!(*keys.borrow(), [key, key]);
        }
    }

    #[test]
    fn held_submit_and_navigation_keys_do_not_repeat() {
        for key in [
            Key::Enter,
            Key::Escape,
            Key::Tab,
            Key::Up,
            Key::Down,
            Key::Home,
            Key::End,
            Key::Other,
        ] {
            clear();
            let keys = record("controls");
            send(key, 30);
            let pressed = keys.borrow().len();
            assert!(!repeat(key, 30));
            assert_eq!(keys.borrow().len(), pressed);
        }
    }

    #[test]
    fn redraw_refreshes_the_handler_without_interrupting_repeat() {
        clear();
        let old = record("redrawn");
        send(Key::Character('a'), 30);
        let refreshed = record("redrawn");
        assert!(repeat(Key::Character('a'), 30));
        assert_eq!(*old.borrow(), [Key::Character('a')]);
        assert_eq!(*refreshed.borrow(), [Key::Character('a')]);
    }

    #[test]
    fn switching_away_and_back_does_not_resume_a_held_key() {
        clear();
        let first = record("first");
        send(Key::Character('a'), 30);
        let second = record("second");
        assert!(!repeat(Key::Character('a'), 30));
        let returned = record("first");
        assert!(!repeat(Key::Character('a'), 30));
        assert_eq!(*first.borrow(), [Key::Character('a')]);
        assert!(second.borrow().is_empty());
        assert!(returned.borrow().is_empty());
    }

    #[test]
    fn clearing_focus_cancels_repeat_even_when_the_same_input_returns() {
        clear();
        record("cleared");
        send(Key::Character('a'), 30);
        clear();
        assert!(!repeat(Key::Character('a'), 30));
        let returned = record("cleared");
        assert!(!repeat(Key::Character('a'), 30));
        assert!(returned.borrow().is_empty());
    }

    #[test]
    fn releasing_another_key_preserves_repeat_but_releasing_the_held_key_stops_it() {
        clear();
        let keys = record("released");
        send(Key::Character('a'), 30);
        release(42);
        assert!(repeat(Key::Character('A'), 30));
        release(30);
        assert!(!repeat(Key::Character('a'), 30));
        assert_eq!(*keys.borrow(), [Key::Character('a'), Key::Character('A')]);
    }

    #[test]
    fn a_key_pressed_before_input_focus_does_not_repeat_into_the_input() {
        clear();
        assert!(!send(Key::Character('a'), 30));
        let keys = record("late-focus");
        assert!(!repeat(Key::Character('a'), 30));
        assert!(keys.borrow().is_empty());
    }

    #[test]
    fn a_repeat_handler_can_clear_focus() {
        clear();
        let received = Rc::new(RefCell::new(0));
        let count = received.clone();
        set(
            "clears-itself",
            Rc::new(move |_| {
                *count.borrow_mut() += 1;
                if *count.borrow() == 2 {
                    clear();
                }
            }),
        );
        send(Key::Character('a'), 30);
        assert!(repeat(Key::Character('a'), 30));
        assert!(!repeat(Key::Character('a'), 30));
        assert_eq!(*received.borrow(), 2);
    }
}
