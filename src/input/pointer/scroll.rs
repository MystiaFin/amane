use crate::Scroll;

use super::Pointer;

impl Pointer {
    pub fn scroll(&self, scroll: Scroll) -> bool {
        let Some(topmost) = self.find_topmost(|target| target.handlers.scroll.is_some()) else {
            return false;
        };

        let Some(handler) = &self.targets[topmost].handlers.scroll else {
            return false;
        };

        handler(scroll);

        true
    }
}
