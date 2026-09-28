use crate::Button;

use super::Pointer;

impl Pointer {
    pub fn press(&mut self) {
        self.pressed = self.find_topmost(|target| target.handlers.click.is_some());
    }

    // a click only counts when the button is let go over the widget it was pressed on
    pub fn release(&mut self, button: Button) -> bool {
        let Some(pressed) = self.pressed.take() else {
            return false;
        };

        let released = self.find_topmost(|target| target.handlers.click.is_some());

        if released != Some(pressed) {
            return false;
        }

        let Some(click) = &self.targets[pressed].handlers.click else {
            return false;
        };

        click(button);

        true
    }
}
