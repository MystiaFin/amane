mod lines;

use ttf_parser::Face;

use crate::graphics::{Rect, Renderer, font};
use crate::{Color, Size, Weight};

use super::Widget;

use lines::Rules;

pub struct Text {
    content: String,
    size: f32,
    color: Color,
    font: Option<String>,
    weight: Weight,
    rules: Rules,
}

impl Text {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            size: 16.0,
            color: Color::BLACK,
            font: None,
            weight: Weight::Regular,
            rules: Rules::default(),
        }
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size;

        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;

        self
    }

    pub fn font(mut self, family: &str) -> Self {
        self.font = Some(String::from(family));

        self
    }

    pub fn weight(mut self, weight: impl Into<Weight>) -> Self {
        self.weight = weight.into();

        self
    }

    pub fn wrap(mut self) -> Self {
        self.rules.wrap = true;

        self
    }

    pub fn elide(mut self) -> Self {
        self.rules.elide = true;

        self
    }

    pub fn max_lines(mut self, count: usize) -> Self {
        self.rules.max_lines = Some(count);

        self
    }

    fn face(&self) -> &'static Face<'static> {
        font::load_weighted(self.font.as_deref(), self.weight)
    }
}

impl Widget for Text {
    fn width(&self) -> Size {
        // wrapped or elided text takes the width it is given and fits itself into it
        if self.rules.wrap || self.rules.elide {
            return Size::Parent;
        }

        Size::Fixed(lines::measure(&self.content, self.face(), self.size))
    }

    fn height(&self) -> Size {
        /*
         * the width is only known when drawing,
         * so wrapped text without a line limit cannot know its height yet
         */
        let count = match (self.rules.wrap, self.rules.max_lines) {
            (false, _) => 1,
            (true, Some(count)) => count,
            (true, None) => return Size::Parent,
        };

        Size::Fixed(lines::stack_height(self.face(), self.size, count))
    }

    fn draw(&self, renderer: &mut Renderer, area: Rect) {
        let font = self.face();

        let arranged = lines::arrange(&self.content, font, self.size, area.width, self.rules);

        let line_height = lines::height(font, self.size);
        let spacing = lines::spacing(font, self.size);

        let mut top = area.y;

        for line in arranged {
            let line_area = Rect::new(area.x, top, area.width, line_height);

            renderer.text(&line, font, self.size, self.color, line_area);

            top += spacing;
        }
    }
}
