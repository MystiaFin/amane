use crate::{InputArea, LayerWindow};

impl LayerWindow {
    // only these areas take the pointer, everywhere else clicks go to the windows below
    pub fn input_region(mut self, areas: Vec<InputArea>) -> Self {
        self.input_region = Some(areas);

        self
    }

    // no area takes the pointer, so every click goes to the windows below
    pub fn click_through(self) -> Self {
        self.input_region(Vec::new())
    }
}
