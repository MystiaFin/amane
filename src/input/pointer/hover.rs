use super::Pointer;

impl Pointer {
    pub fn move_to(&mut self, x: f32, y: f32) -> bool {
        self.x = x;
        self.y = y;

        let mut inside = Vec::new();

        for (index, target) in self.targets.iter().enumerate() {
            if target.handlers.hover.is_some() && target.area.contains(x, y) {
                inside.push(index);
            }
        }

        self.update_hovered(inside)
    }

    pub fn leave(&mut self) -> bool {
        // a button let go outside the window is not a click
        self.pressed = None;

        // the release will not reach this window, so the drag ends here
        self.dragged = None;

        self.update_hovered(Vec::new())
    }

    // tells the targets the pointer entered or left, and returns whether any were told
    fn update_hovered(&mut self, inside: Vec<usize>) -> bool {
        let mut changed = false;

        for &index in &self.hovered {
            if !inside.contains(&index) {
                changed |= self.notify(index, false);
            }
        }

        for &index in &inside {
            if !self.hovered.contains(&index) {
                changed |= self.notify(index, true);
            }
        }

        self.hovered = inside;

        changed
    }

    fn notify(&self, index: usize, inside: bool) -> bool {
        // a redraw since the pointer entered may have removed this target
        let Some(target) = self.targets.get(index) else {
            return false;
        };

        let Some(hover) = &target.handlers.hover else {
            return false;
        };

        hover(inside);

        true
    }
}
