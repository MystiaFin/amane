use std::rc::Rc;

use crate::Scroll;
use crate::graphics::Area;
use crate::input::{Handlers, PIXELS_PER_LINE, Target, clip};

use super::{ScrollArea, offsets};

pub fn collect_targets(scroll_area: &ScrollArea, area: Area, targets: &mut Vec<Target>) {
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
    targets.push(Target::new(area, handlers));

    let first = targets.len();

    scroll_area
        .child
        .collect_targets(scroll_area.child_area(area), targets);

    // the parts scrolled out of view don't react to the pointer
    clip(&mut targets[first..], area);
}
