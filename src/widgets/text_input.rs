mod field;
mod fields;
mod input;

use std::rc::Rc;

use crate::graphics::{Rect, Renderer};
use crate::input::{Target, focus};
use crate::{Color, Size, Text};

use super::Widget;

use field::Field;

// shared, so the key handler kept after a redraw still reaches it
pub type TextHandler = Rc<dyn Fn(String)>;

const CURSOR_WIDTH: f32 = 1.5;

pub struct TextInput {
    // the view builds a fresh input on every redraw, so the text is kept under this name
    id: &'static str,

    placeholder: String,
    size: f32,
    color: Color,
    width: Size,

    // shows a dot in place of each letter
    password: bool,

    // takes the keyboard as soon as it is drawn, without a click
    focused: bool,

    on_change: Option<TextHandler>,
    on_submit: Option<TextHandler>,
}

impl TextInput {
    pub fn new(id: &'static str) -> Self {
        Self {
            id,
            placeholder: String::new(),
            size: 16.0,
            color: Color::BLACK,
            width: Size::Parent,
            password: false,
            focused: false,
            on_change: None,
            on_submit: None,
        }
    }

    pub fn width(mut self, width: impl Into<Size>) -> Self {
        self.width = width.into();

        self
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size;

        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;

        self
    }

    // shown faded while the input is empty
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();

        self
    }

    pub fn password(mut self) -> Self {
        self.password = true;

        self
    }

    pub fn focused(mut self) -> Self {
        self.focused = true;

        self
    }

    // replaces what the input holds, with the cursor at the end
    pub fn set_text(id: &'static str, text: &str) {
        let field = Field {
            text: String::from(text),
            cursor: text.chars().count(),
        };

        fields::set(id, field);
    }

    // runs after every change to the text, with the whole new text
    pub fn on_change(mut self, handler: impl Fn(String) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));

        self
    }

    // runs when Enter is pressed
    pub fn on_submit(mut self, handler: impl Fn(String) + 'static) -> Self {
        self.on_submit = Some(Rc::new(handler));

        self
    }

    // the text as it is drawn, which hides a password
    fn shown(&self, text: &str) -> String {
        if !self.password {
            return String::from(text);
        }

        "•".repeat(text.chars().count())
    }

    fn label(&self, content: impl Into<String>, color: Color) -> Text {
        Text::new(content).size(self.size).color(color)
    }

    fn draw_cursor(&self, renderer: &mut Renderer, area: Rect) {
        let field = fields::get(self.id);

        let before = self.label(self.shown(&field.before_cursor()), self.color);
        let offset = before.width().resolve(area.width);

        let cursor = Rect::new(area.x + offset, area.y, CURSOR_WIDTH, area.height);

        renderer.rectangle(cursor, self.color, 0.0);
    }
}

impl Widget for TextInput {
    fn width(&self) -> Size {
        self.width
    }

    fn height(&self) -> Size {
        self.label("", self.color).height()
    }

    fn draw(&self, renderer: &mut Renderer, area: Rect) {
        let text = fields::get(self.id).text;

        // the placeholder is the text color at half strength
        let faded = Color::rgba(self.color.r, self.color.g, self.color.b, self.color.a / 2);

        let label = if text.is_empty() {
            self.label(self.placeholder.as_str(), faded)
        } else {
            self.label(self.shown(&text), self.color)
        };

        // text longer than the input is cut off at its edge
        let mut inside = renderer.layer();

        label.draw(&mut inside, area);

        if focus::has(self.id) {
            self.draw_cursor(&mut inside, area);
        }

        renderer.clip(inside, area, 0.0);
    }

    fn collect_targets(&self, area: Rect, targets: &mut Vec<Target>) {
        input::collect_targets(self, area, targets);
    }
}
