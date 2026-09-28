use crate::Key;

// what a text input holds: its text, and where the cursor sits in it
#[derive(Clone, Default)]
pub struct Field {
    pub text: String,

    // counts letters, not bytes
    pub cursor: usize,
}

impl Field {
    // returns whether the text changed, moving the cursor alone does not count
    pub fn apply(&mut self, key: Key) -> bool {
        match key {
            Key::Character(letter) => self.insert(letter),
            Key::Space => self.insert(' '),
            Key::Backspace => self.erase(),
            Key::Left => self.move_to(self.cursor.saturating_sub(1)),
            Key::Right => self.move_to(self.cursor + 1),
            Key::Home => self.move_to(0),
            Key::End => self.move_to(self.length()),
            _ => false,
        }
    }

    pub fn before_cursor(&self) -> String {
        self.text.chars().take(self.cursor).collect()
    }

    fn insert(&mut self, letter: char) -> bool {
        let at = self.byte_index();

        self.text.insert(at, letter);
        self.cursor += 1;

        true
    }

    // removes the letter left of the cursor
    fn erase(&mut self) -> bool {
        if self.cursor == 0 {
            return false;
        }

        self.cursor -= 1;

        let at = self.byte_index();

        self.text.remove(at);

        true
    }

    fn move_to(&mut self, cursor: usize) -> bool {
        self.cursor = cursor.min(self.length());

        false
    }

    fn length(&self) -> usize {
        self.text.chars().count()
    }

    // a String is indexed by bytes, and one letter can take several
    fn byte_index(&self) -> usize {
        match self.text.char_indices().nth(self.cursor) {
            Some((index, _)) => index,
            None => self.text.len(),
        }
    }
}
