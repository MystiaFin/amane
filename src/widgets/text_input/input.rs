use std::rc::Rc;

use crate::graphics::Rect;
use crate::input::{Handlers, KeyHandler, Target, focus};
use crate::{Button, Key};

use super::{TextHandler, TextInput, fields};

pub fn collect_targets(text_input: &TextInput, area: Rect, targets: &mut Vec<Target>) {
    let id = text_input.id;
    let on_key = key_handler(text_input);

    // the handler holds this redraw's on_change and on_submit, so a focused input takes the new one
    if focus::has(id) {
        focus::set(id, on_key.clone());
    }

    let on_click = move |_: Button| {
        focus::set(id, on_key.clone());
    };

    let handlers = Handlers {
        click: Some(Rc::new(on_click)),
        ..Handlers::default()
    };

    targets.push(Target { area, handlers });
}

fn key_handler(text_input: &TextInput) -> KeyHandler {
    let id = text_input.id;

    let on_change = text_input.on_change.clone();
    let on_submit = text_input.on_submit.clone();

    let on_key = move |key: Key| match key {
        Key::Enter => submit(id, on_submit.as_ref()),
        Key::Escape => focus::clear(),
        _ => edit(id, key, on_change.as_ref()),
    };

    Rc::new(on_key)
}

fn submit(id: &'static str, on_submit: Option<&TextHandler>) {
    let Some(on_submit) = on_submit else {
        return;
    };

    on_submit(fields::get(id).text);
}

fn edit(id: &'static str, key: Key, on_change: Option<&TextHandler>) {
    let mut field = fields::get(id);

    let changed = field.apply(key);
    let text = field.text.clone();

    // stored before the handler runs, so it reads the new text
    fields::set(id, field);

    if !changed {
        return;
    }

    let Some(on_change) = on_change else {
        return;
    };

    on_change(text);
}
