use std::rc::Rc;

use crate::Scroll;
use crate::graphics::Rect;
use crate::input::{Handlers, Target, clip};

use super::{ScrollArea, offsets};

// the rate the backend turns touchpad pixels into lines, so a touchpad moves the list 1:1
const PIXELS_PER_LINE: f32 = 15.0;

pub fn collect_targets(scroll_area: &ScrollArea, area: Rect, targets: &mut Vec<Target>) {
    let id = scroll_area.id;
    let farthest = scroll_area.farthest(area);

    let on_scroll = move |scroll: Scroll| {
        let current = offsets::get(id).clamp(0.0, farthest);
        let moved = scroll.y * PIXELS_PER_LINE;

        let offset = (current + moved).clamp(0.0, farthest);

        offsets::set(id, offset);
    };

    let handlers = Handlers {
        scroll: Some(Rc::new(on_scroll)),
        ..Handlers::default()
    };

    // added before the child, so a child with its own on_scroll still wins
    targets.push(Target { area, handlers });

    let first = targets.len();

    scroll_area
        .child
        .collect_targets(scroll_area.child_area(area), targets);

    // the parts scrolled out of view don't react to the pointer
    clip(&mut targets[first..], area);
}
