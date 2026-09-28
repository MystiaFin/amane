use std::cell::RefCell;
use std::collections::HashMap;

use super::field::Field;

thread_local! {
    // what each text input holds, kept apart from the widgets that are rebuilt each redraw
    static FIELDS: RefCell<HashMap<&'static str, Field>> = RefCell::new(HashMap::new());
}

pub fn get(id: &'static str) -> Field {
    FIELDS.with_borrow(|fields| fields.get(id).cloned().unwrap_or_default())
}

pub fn set(id: &'static str, field: Field) {
    FIELDS.with_borrow_mut(|fields| {
        fields.insert(id, field);
    });
}
