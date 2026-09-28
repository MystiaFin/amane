use amane::{App, Color, Column, LayerWindow, Parent, Rectangle, ScrollArea, Text, Widget};

fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    LayerWindow::new().width(300.0).height(400.0).child(
        Rectangle::new()
            .width(Parent)
            .height(Parent)
            .radius(24.0)
            .fill(Color::WHITE)
            .clip()
            .child(ScrollArea::new("items", Column::new(items()))),
    )
}

// taller than the window, so the list has somewhere to scroll
fn items() -> Vec<Box<dyn Widget>> {
    let mut items: Vec<Box<dyn Widget>> = Vec::new();

    for number in 1..=30 {
        let label = format!("item {number}");

        // every other row is shaded, so the movement is easy to see
        let shade = if number % 2 == 0 {
            Color::WHITE
        } else {
            Color::from("#e8e8e8")
        };

        let item = Rectangle::new()
            .width(Parent)
            .height(40.0)
            .fill(shade)
            .on_click(move |_| println!("clicked item {number}"))
            .child(Text::new(label));

        items.push(Box::new(item));
    }

    items
}
