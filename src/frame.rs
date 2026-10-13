use std::any::TypeId;
use std::collections::HashSet;

use crate::Widget;
use crate::animation::moving;
use crate::changes;
use crate::graphics::{Area, Renderer, Transform};
use crate::input::Target;
use crate::scale::ScaleFactor;

// one window's frame: what the gpu draws, and where the widgets take the pointer
pub struct Frame {
    pub renderer: Renderer,
    pub targets: Vec<Target>,

    // services read while drawing, on top of the ones the view read
    pub reads: HashSet<TypeId>,

    // something the view or a widget drew has not arrived yet, so another frame is needed
    pub moving: bool,
}

/*
 * runs a window's view at the window's size; the services it read come
 * back with it, so a later change to one of them draws the window again
 */
pub fn run_view<T>(
    view: impl FnOnce() -> T,
    width: u32,
    height: u32,
    scale_factor: ScaleFactor,
) -> (T, HashSet<TypeId>) {
    // anything read or set moving before belongs to another window
    changes::take_read();

    moving::take();

    crate::window::set_size(
        scale_factor.logical(width as f32),
        scale_factor.logical(height as f32),
    );

    // the view runs again on every redraw, so it shows the services as they are now
    let content = view();

    let reads = changes::take_read();

    (content, reads)
}

// lays the root out in the window, draws it, and collects where it reacts to the pointer
pub fn build(
    root: &dyn Widget,
    width: u32,
    height: u32,
    scale: f32,
    scale_factor: ScaleFactor,
) -> Frame {
    let area = root_area(
        root,
        scale_factor.logical(width as f32),
        scale_factor.logical(height as f32),
    );

    let mut renderer = Renderer::new(scale * scale_factor.get());

    root.draw(&mut renderer, area);

    let reads = changes::take_read();

    // read after drawing, since a shader that runs on time sets it while drawing
    let moving = moving::take();

    let mut targets = Vec::new();

    root.collect_targets(area, &mut targets);

    // pointer positions use surface units; undo the global scale before each widget's transform
    let inverse = Transform::from_scale(scale_factor.logical(1.0), scale_factor.logical(1.0));

    for target in &mut targets {
        target.inverse = inverse.post_concat(target.inverse);
    }

    Frame {
        renderer,
        targets,
        reads,
        moving,
    }
}

// the root at its own size, from the window's top left corner
fn root_area(root: &dyn Widget, width: f32, height: f32) -> Area {
    let root_width = root.width().resolve(width);
    let root_height = root.height().resolve(height);

    Area::new(0.0, 0.0, root_width, root_height)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::Pointer;
    use crate::{Center, Column, End, Justify, Parent, Rectangle, Row, Stack, children};
    use std::cell::Cell;
    use std::rc::Rc;

    // where each rectangle landed, in the order the tree lists them
    fn areas(root: &dyn Widget, width: u32, height: u32) -> Vec<Area> {
        let frame = build(root, width, height, 1.0, ScaleFactor::default());

        let mut areas = Vec::new();

        for target in frame.targets {
            areas.push(target.area);
        }

        areas
    }

    fn block(width: f32, height: f32) -> Rectangle {
        Rectangle::new().width(width).height(height)
    }

    #[test]
    fn scaled_views_measure_the_window_in_widget_units() {
        let (size, _) = run_view(crate::window_size, 300, 150, ScaleFactor::new(1.5));

        assert_eq!(size, (200.0, 100.0));
    }

    #[test]
    fn global_scale_keeps_parent_widgets_filling_the_window() {
        let root = Rectangle::new()
            .width(Parent)
            .height(Parent)
            .fill(crate::Color::BLUE);
        let frame = build(&root, 300, 150, 1.25, ScaleFactor::new(1.5));

        assert_eq!(frame.targets[0].area, Area::new(0.0, 0.0, 200.0, 100.0));

        assert!(frame.targets[0].contains(299.0, 149.0));
        assert!(!frame.targets[0].contains(301.0, 151.0));
    }

    #[test]
    fn scaled_pointer_undoes_global_scale_before_widget_transforms() {
        let moved = Rc::new(Cell::new(crate::Point::default()));
        let dragged = Rc::new(Cell::new(crate::Point::default()));
        let root = block(40.0, 20.0)
            .translate(30.0, 10.0)
            .on_move({
                let moved = moved.clone();
                move |point| moved.set(point)
            })
            .on_drag({
                let dragged = dragged.clone();
                move |point| dragged.set(point)
            });

        // Wayland pointer positions use surface units, regardless of output DPI.
        let frame = build(&root, 200, 100, 1.25, ScaleFactor::new(2.0));
        let mut pointer = Pointer::default();
        pointer.set_targets(frame.targets);
        pointer.move_to(80.0, 40.0);

        assert!(pointer.report_motion());
        assert!(pointer.start_drag());
        assert_eq!(moved.get(), crate::Point { x: 10.0, y: 10.0 });
        assert_eq!(dragged.get(), moved.get());

        pointer.move_to(160.0, 60.0);
        assert!(pointer.drag());
        assert_eq!(dragged.get(), crate::Point { x: 50.0, y: 20.0 });
    }

    #[test]
    fn row_gives_the_rest_to_parent_sized_children() {
        let row = Row::new(children![
            block(100.0, 20.0),
            Rectangle::new().width(Parent).height(20.0),
        ])
        .width(Parent);

        let placed = areas(&row, 400, 50);

        assert_eq!(placed[0], Area::new(0.0, 0.0, 100.0, 20.0));
        assert_eq!(placed[1], Area::new(100.0, 0.0, 300.0, 20.0));
    }

    #[test]
    fn column_centers_and_spreads() {
        let centered = Column::new(children![block(10.0, 10.0), block(10.0, 10.0)])
            .width(Parent)
            .height(Parent)
            .justify(Center)
            .align(End);

        let placed = areas(&centered, 30, 100);

        // 80 free pixels, half of them above the first child; End pushes them right
        assert_eq!(placed[0], Area::new(20.0, 40.0, 10.0, 10.0));
        assert_eq!(placed[1], Area::new(20.0, 50.0, 10.0, 10.0));

        let spread = Row::new(children![
            block(10.0, 10.0),
            block(10.0, 10.0),
            block(10.0, 10.0)
        ])
        .width(Parent)
        .justify(Justify::SpaceBetween);

        let placed = areas(&spread, 100, 10);

        assert_eq!(placed[1].x, 45.0);
        assert_eq!(placed[2].x, 90.0);
    }

    #[test]
    fn stack_puts_children_at_the_same_corner() {
        let stack = Stack::new(children![block(50.0, 50.0), block(20.0, 30.0)]);

        let placed = areas(&stack, 200, 200);

        assert_eq!(placed[0], Area::new(0.0, 0.0, 50.0, 50.0));
        assert_eq!(placed[1], Area::new(0.0, 0.0, 20.0, 30.0));
    }

    #[test]
    fn rectangle_places_its_child_inside_the_padding() {
        let card = block(100.0, 60.0)
            .padding(10.0)
            .align_child(Center, End)
            .child(block(20.0, 20.0));

        let placed = areas(&card, 100, 60);

        assert_eq!(placed[0], Area::new(0.0, 0.0, 100.0, 60.0));

        // 80 by 40 inside the padding: centered across, at the bottom down
        assert_eq!(placed[1], Area::new(40.0, 30.0, 20.0, 20.0));
    }

    // like a rectangle whose shader reads time
    struct Ticking;

    impl Widget for Ticking {
        fn width(&self) -> crate::Size {
            Parent
        }

        fn height(&self) -> crate::Size {
            Parent
        }

        fn draw(&self, _: &mut Renderer, _: Area) {
            moving::set();
        }
    }

    #[test]
    fn sees_motion_started_while_drawing() {
        let (_, _) = run_view(|| (), 10, 10, ScaleFactor::default());

        assert!(build(&Ticking, 10, 10, 1.0, ScaleFactor::default()).moving);

        // taken by that frame, so the next window starts still
        assert!(!build(&block(1.0, 1.0), 10, 10, 1.0, ScaleFactor::default()).moving);
    }
}
