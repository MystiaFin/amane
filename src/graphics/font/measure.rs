use std::cell::RefCell;

use swash::FontRef;
use swash::scale::ScaleContext;

use super::{fallback, sliders};

thread_local! {
    // swash keeps its working memory here, so each letter doesn't allocate it again
    static CONTEXT: RefCell<ScaleContext> = RefCell::new(ScaleContext::new());
}

// swash's working memory for turning letters into outlines and pixels
pub fn with_context<T>(work: impl FnOnce(&mut ScaleContext) -> T) -> T {
    CONTEXT.with_borrow_mut(work)
}

// one letter at a text size: the font that has it, its glyph, and how it measures in pixels
pub struct MeasuredLetter {
    pub font: FontRef<'static>,
    pub id: u16,

    // font units to pixels; a fallback font can measure in other units than the text's own
    pub units: f32,

    // how far the letter moves the pen
    pub advance: f32,
}

pub fn measure_letter(font: FontRef<'static>, letter: char, size: f32) -> MeasuredLetter {
    let (font, id) = fallback::with_letter(font, letter);

    let units_per_em = font.metrics(sliders::coords(font)).units_per_em;

    let units = size / f32::from(units_per_em);

    let advance = font.glyph_metrics(sliders::coords(font)).advance_width(id);

    MeasuredLetter {
        font,
        id,
        units,
        advance: advance * units,
    }
}
