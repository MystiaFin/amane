use std::cell::RefCell;

use swash::{CacheKey, FontRef};

use crate::cache::Cache;

use super::lines::{self, Rules};

/*
 * the view builds every text again on each redraw, so the same text gets
 * measured letter by letter again and again; these cache the results
 */

// a text measured before: the same letters, font and size always measure the same
#[derive(PartialEq, Eq, Hash)]
struct MeasuredText {
    letters: String,

    // which font file, as swash numbers them
    font: CacheKey,

    // the size as raw bits, since a decimal number can't be looked up exactly
    size_bits: u32,
}

// a text split into lines to fit a width, which also changes where the lines break
#[derive(PartialEq, Eq, Hash)]
struct FittedText {
    text: MeasuredText,
    width_bits: u32,
    rules: Rules,
}

/*
 * thread_local gives every thread its own copy, so no lock is needed;
 * only the main thread draws, so in practice there is one of each.
 * RefCell lets the shared cache change, which a plain static can't
 */
thread_local! {
    static WIDTHS: RefCell<Cache<MeasuredText, f32>> = RefCell::new(Cache::new());

    // where the letters' ink starts and ends, for text sized by its shapes
    static INK_EDGES: RefCell<Cache<MeasuredText, (f32, f32)>> = RefCell::new(Cache::new());

    static LINES: RefCell<Cache<FittedText, Vec<String>>> = RefCell::new(Cache::new());
}

impl MeasuredText {
    fn new(letters: &str, font: FontRef<'static>, size: f32) -> Self {
        Self {
            letters: String::from(letters),
            font: font.key,
            size_bits: size.to_bits(),
        }
    }
}

pub fn measure(letters: &str, font: FontRef<'static>, size: f32) -> f32 {
    let question = MeasuredText::new(letters, font, size);

    let work_out = || lines::measure(letters, font, size);

    WIDTHS.with_borrow_mut(|widths| widths.look_up(question, work_out))
}

pub fn find_ink(letters: &str, font: FontRef<'static>, size: f32) -> (f32, f32) {
    let question = MeasuredText::new(letters, font, size);

    let work_out = || lines::ink(letters, font, size);

    INK_EDGES.with_borrow_mut(|edges| edges.look_up(question, work_out))
}

pub fn split_into_lines(
    letters: &str,
    font: FontRef<'static>,
    size: f32,
    width: f32,
    rules: Rules,
) -> Vec<String> {
    let question = FittedText {
        text: MeasuredText::new(letters, font, size),
        width_bits: width.to_bits(),
        rules,
    };

    let work_out = || lines::arrange(letters, font, size, width, rules);

    LINES.with_borrow_mut(|split| split.look_up(question, work_out))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Weight;
    use crate::graphics::font;

    #[test]
    fn reuses_width() {
        let font = font::load_weighted(None, Weight::Regular);

        let wrapped = Rules {
            wrap: true,
            ..Rules::default()
        };

        // the first round measures and caches, the second reads from the cache
        for _ in 0..2 {
            let width = measure("hello world", font, 16.0);

            assert_eq!(width, lines::measure("hello world", font, 16.0));

            let edges = find_ink("hello world", font, 16.0);

            assert_eq!(edges, lines::ink("hello world", font, 16.0));

            let split = split_into_lines("hello there world", font, 16.0, 60.0, wrapped);
            let fresh = lines::arrange("hello there world", font, 16.0, 60.0, wrapped);

            assert_eq!(split, fresh);
        }
    }

    #[test]
    fn width_follows_size() {
        let font = font::load_weighted(None, Weight::Regular);

        // the same letters at two sizes, so the 16px width must not be reused for 32px
        let small = measure("hello", font, 16.0);
        let big = measure("hello", font, 32.0);

        assert!(big > small);
    }
}
