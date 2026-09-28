use crate::input::Cursor;

use super::Pointer;

impl Pointer {
    // the look asked for by the innermost target under the pointer
    pub fn cursor(&self) -> Cursor {
        let Some(index) = self.find_topmost(|target| target.handlers.cursor.is_some()) else {
            return Cursor::Default;
        };

        self.targets[index].handlers.cursor.unwrap_or(Cursor::Default)
    }
}
