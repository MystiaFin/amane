use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use smithay_client_toolkit::reexports::calloop::{
    LoopHandle, RegistrationToken,
    timer::{TimeoutAction, Timer},
};
use smithay_client_toolkit::seat::keyboard::{KeyEvent, RawModifiers, RepeatInfo};
use xkbcommon::xkb;

pub struct Repeat {
    info: RepeatInfo,
    event: Rc<RefCell<Option<KeyEvent>>>,
    timer: Option<RegistrationToken>,
    keymap: Option<xkb::State>,
}

impl Default for Repeat {
    fn default() -> Self {
        Self {
            info: RepeatInfo::Disable,
            event: Rc::default(),
            timer: None,
            keymap: None,
        }
    }
}

impl Repeat {
    pub fn keymap(&mut self, text: &str) {
        let context = xkb::Context::new(xkb::CONTEXT_NO_FLAGS);
        self.keymap = xkb::Keymap::new_from_string(
            &context,
            text.to_owned(),
            xkb::KEYMAP_FORMAT_TEXT_V1,
            xkb::COMPILE_NO_FLAGS,
        )
        .map(|keymap| xkb::State::new(&keymap));
    }

    pub fn native(&self, mut event: KeyEvent) -> KeyEvent {
        // SCTK's compositor-generated repeats have no UTF-8 text.
        if event.utf8.is_none()
            && let Some(state) = &self.keymap
        {
            update_text(state, &mut event);
        }
        event
    }

    pub fn modifiers(&mut self, modifiers: RawModifiers, group: u32) {
        if let Some(state) = &mut self.keymap {
            state.update_mask(
                modifiers.depressed,
                modifiers.latched,
                modifiers.locked,
                0,
                0,
                group,
            );
            if let Some(event) = self.event.borrow_mut().as_mut() {
                update_text(state, event);
            }
        }
    }

    pub fn configure<D: 'static>(
        &mut self,
        info: RepeatInfo,
        handle: &LoopHandle<'static, D>,
        callback: fn(&mut D, KeyEvent) -> bool,
    ) {
        self.stop(handle);
        self.info = info;
        self.arm(handle, callback);
    }

    pub fn press<D: 'static>(
        &mut self,
        event: KeyEvent,
        enabled: bool,
        handle: &LoopHandle<'static, D>,
        callback: fn(&mut D, KeyEvent) -> bool,
    ) {
        let repeats = self.keymap.as_ref().is_some_and(|state| {
            state
                .get_keymap()
                .key_repeats(xkb::Keycode::new(event.raw_code + 8))
        });
        if !repeats {
            return;
        }
        self.cancel(handle);
        if enabled {
            self.event.replace(Some(event));
            self.arm(handle, callback);
        }
    }

    pub fn release<D>(&mut self, raw_code: u32, handle: &LoopHandle<'static, D>) {
        let held = self.event.borrow().as_ref().map(|event| event.raw_code);
        if held == Some(raw_code) {
            self.cancel(handle);
        }
    }

    pub fn cancel<D>(&mut self, handle: &LoopHandle<'static, D>) {
        self.stop(handle);
        self.event.replace(None);
    }

    fn stop<D>(&mut self, handle: &LoopHandle<'static, D>) {
        if let Some(timer) = self.timer.take() {
            handle.remove(timer);
        }
    }

    fn arm<D: 'static>(
        &mut self,
        handle: &LoopHandle<'static, D>,
        callback: fn(&mut D, KeyEvent) -> bool,
    ) {
        let RepeatInfo::Repeat { rate, delay } = self.info else {
            return;
        };
        if self.event.borrow().is_none() {
            return;
        }
        let event = self.event.clone();
        let gap = Duration::from_secs_f64(1.0 / f64::from(rate.get()));
        let timer = Timer::from_duration(Duration::from_millis(u64::from(delay)));
        self.timer = Some(
            handle
                .insert_source(timer, move |_, _, state| {
                    let current = event.borrow().clone();
                    let Some(current) = current else {
                        return TimeoutAction::Drop;
                    };
                    if callback(state, current) {
                        TimeoutAction::ToDuration(gap)
                    } else {
                        event.replace(None);
                        TimeoutAction::Drop
                    }
                })
                .expect("failed to insert keyboard repeat timer"),
        );
    }
}

fn update_text(state: &xkb::State, event: &mut KeyEvent) {
    let key = xkb::Keycode::new(event.raw_code + 8);
    event.keysym = state.key_get_one_sym(key);
    event.utf8 = Some(state.key_get_utf8(key));
}

#[cfg(test)]
mod tests {
    use super::*;
    use smithay_client_toolkit::reexports::calloop::EventLoop;
    use std::num::NonZeroU32;

    fn setup() -> (Repeat, EventLoop<'static, Vec<String>>) {
        let context = xkb::Context::new(xkb::CONTEXT_NO_FLAGS);
        let keymap = xkb::Keymap::new_from_names(
            &context,
            "evdev",
            "pc105",
            "us",
            "",
            None,
            xkb::COMPILE_NO_FLAGS,
        )
        .unwrap();
        let mut repeat = Repeat::default();
        repeat.keymap(&keymap.get_as_string(xkb::KEYMAP_FORMAT_TEXT_V1));
        let event_loop = EventLoop::try_new().unwrap();
        repeat.configure(
            RepeatInfo::Repeat {
                rate: NonZeroU32::new(500).unwrap(),
                delay: 0,
            },
            &event_loop.handle(),
            record,
        );
        (repeat, event_loop)
    }
    fn event() -> KeyEvent {
        KeyEvent {
            time: 0,
            raw_code: 30,
            keysym: super::super::Keysym::a,
            utf8: Some("a".into()),
        }
    }
    fn record(text: &mut Vec<String>, event: KeyEvent) -> bool {
        text.push(event.utf8.unwrap());
        true
    }
    #[test]
    fn native_repeat_recovers_text_from_the_current_keymap() {
        let (repeat, _) = setup();
        let mut native = event();
        native.utf8 = None;
        assert_eq!(repeat.native(native).utf8.as_deref(), Some("a"));
    }
    #[test]
    fn modifier_changes_update_a_held_character() {
        let (mut repeat, mut event_loop) = setup();
        let mut text = Vec::new();
        repeat.press(event(), true, &event_loop.handle(), record);
        repeat.modifiers(
            RawModifiers {
                depressed: 1,
                latched: 0,
                locked: 0,
            },
            0,
        );
        event_loop
            .dispatch(Some(Duration::from_millis(5)), &mut text)
            .unwrap();
        assert_eq!(text, ["A"]);
    }
    #[test]
    fn releasing_a_held_key_removes_its_timer() {
        let (mut repeat, mut event_loop) = setup();
        let mut text = Vec::new();
        repeat.press(event(), true, &event_loop.handle(), record);
        event_loop
            .dispatch(Some(Duration::from_millis(5)), &mut text)
            .unwrap();
        assert_eq!(text, ["a"]);
        repeat.release(30, &event_loop.handle());
        event_loop
            .dispatch(Some(Duration::from_millis(5)), &mut text)
            .unwrap();
        assert_eq!(text, ["a"]);
    }
    #[test]
    fn cancelling_repeat_removes_its_timer() {
        let (mut repeat, mut event_loop) = setup();
        let mut text = Vec::new();
        repeat.press(event(), true, &event_loop.handle(), record);
        event_loop
            .dispatch(Some(Duration::from_millis(5)), &mut text)
            .unwrap();
        assert_eq!(text, ["a"]);
        repeat.cancel(&event_loop.handle());
        event_loop
            .dispatch(Some(Duration::from_millis(5)), &mut text)
            .unwrap();
        assert_eq!(text, ["a"]);
    }
    #[test]
    fn disabling_repeat_during_a_hold_stops_existing_events() {
        let (mut repeat, mut event_loop) = setup();
        let mut text = Vec::new();
        repeat.press(event(), true, &event_loop.handle(), record);
        event_loop
            .dispatch(Some(Duration::from_millis(5)), &mut text)
            .unwrap();
        assert_eq!(text, ["a"]);
        repeat.configure(RepeatInfo::Disable, &event_loop.handle(), record);
        event_loop
            .dispatch(Some(Duration::from_millis(5)), &mut text)
            .unwrap();
        assert_eq!(text, ["a"]);
    }
    #[test]
    fn disabling_repeat_before_the_delay_prevents_the_first_event() {
        let (mut repeat, mut event_loop) = setup();
        let mut text = Vec::new();
        repeat.configure(
            RepeatInfo::Repeat {
                rate: NonZeroU32::new(500).unwrap(),
                delay: 20,
            },
            &event_loop.handle(),
            record,
        );
        repeat.press(event(), true, &event_loop.handle(), record);
        repeat.configure(RepeatInfo::Disable, &event_loop.handle(), record);
        event_loop
            .dispatch(Some(Duration::from_millis(30)), &mut text)
            .unwrap();
        assert!(text.is_empty());
    }
    #[test]
    fn rejecting_a_repeat_target_removes_the_timer() {
        let (mut repeat, mut event_loop) = setup();
        let mut text = Vec::new();
        repeat.press(event(), true, &event_loop.handle(), |text, event| {
            text.push(event.utf8.unwrap());
            false
        });
        event_loop
            .dispatch(Some(Duration::from_millis(5)), &mut text)
            .unwrap();
        event_loop
            .dispatch(Some(Duration::from_millis(5)), &mut text)
            .unwrap();
        assert_eq!(text, ["a"]);
    }
    #[test]
    fn keys_disabled_in_the_keymap_do_not_start_repeat() {
        let (mut repeat, mut event_loop) = setup();
        let mut text = Vec::new();
        let mut modifier = event();
        modifier.raw_code = 42;
        modifier.keysym = super::super::Keysym::Shift_L;
        modifier.utf8 = None;
        repeat.press(modifier, true, &event_loop.handle(), record);
        event_loop
            .dispatch(Some(Duration::from_millis(5)), &mut text)
            .unwrap();
        assert!(text.is_empty());
    }
}
