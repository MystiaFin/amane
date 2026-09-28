use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    // how far each scroll area has moved, kept apart from the widgets that are rebuilt each redraw
    static OFFSETS: RefCell<HashMap<&'static str, f32>> = RefCell::new(HashMap::new());
}

pub fn get(id: &'static str) -> f32 {
    OFFSETS.with_borrow(|offsets| offsets.get(id).copied().unwrap_or(0.0))
}

pub fn set(id: &'static str, offset: f32) {
    OFFSETS.with_borrow_mut(|offsets| {
        offsets.insert(id, offset);
    });
}
