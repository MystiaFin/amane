use std::process;
use std::time::Duration;

use amane::{
    App, Color, Column, Key, Keyboard, LayerWindow, Rectangle, Service, Text, TextInput, Widget,
};

// type to filter, up and down pick, enter prints the pick, escape quits
const FRUITS: [&str; 6] = ["apple", "banana", "cherry", "grape", "lemon", "mango"];

struct Search {
    query: String,
    selected: usize,
}

impl Service for Search {
    fn new() -> Self {
        // starts with a query already typed, to show set_text
        TextInput::set_text("search", "an");

        Self {
            query: String::from("an"),
            selected: 0,
        }
    }

    // nothing changes on its own, only through keys
    fn interval() -> Duration {
        Duration::from_secs(3600)
    }
}

fn main() {
    App::new().window(view).run();
}

fn matches(query: &str) -> Vec<&'static str> {
    let mut found = Vec::new();

    for fruit in FRUITS {
        if fruit.contains(query) {
            found.push(fruit);
        }
    }

    found
}

fn view() -> LayerWindow {
    let search = Search::read();

    let mut rows: Vec<Box<dyn Widget>> = Vec::new();

    // focused, so typing works without clicking the input first
    let input = TextInput::new("search")
        .size(18.0)
        .focused()
        .on_change(|text| {
            let mut search = Search::write();

            search.query = text;
            search.selected = 0;
        })
        .on_submit(|_| submitted());

    rows.push(Box::new(
        Rectangle::new()
            .width(300.0)
            .height(36.0)
            .fill(Color::WHITE)
            .child(input),
    ));

    for (index, fruit) in matches(&search.query).into_iter().enumerate() {
        let fill = if index == search.selected {
            Color::from("#cba6f7")
        } else {
            Color::from("#1e1e2e")
        };

        rows.push(Box::new(
            Rectangle::new()
                .width(300.0)
                .height(30.0)
                .fill(fill)
                .child(Text::new(fruit).size(16.0).color(Color::WHITE)),
        ));
    }

    LayerWindow::new()
        .width(300.0)
        .height(220.0)
        .keyboard(Keyboard::Exclusive)
        .on_key(key_pressed)
        .child(Column::new(rows))
}

// the input keeps letters and enter for itself, arrows and escape reach the window
fn key_pressed(key: Key) {
    let mut search = Search::write();

    let count = matches(&search.query).len();

    match key {
        Key::Down if search.selected + 1 < count => search.selected += 1,
        Key::Up => search.selected = search.selected.saturating_sub(1),
        Key::Escape => process::exit(0),
        _ => {}
    }
}

fn submitted() {
    let search = Search::read();

    let found = matches(&search.query);

    let Some(fruit) = found.get(search.selected) else {
        return;
    };

    println!("picked {fruit}");
}
