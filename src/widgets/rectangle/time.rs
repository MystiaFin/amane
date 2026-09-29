use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

thread_local! {
    // read once per shader file, a shader's source doesn't change while the shell runs
    static READS_TIME: RefCell<HashMap<PathBuf, bool>> = RefCell::new(HashMap::new());
}

/*
 * only a shader that reads time changes between frames, so only that
 * one keeps the window drawing; any `time` word counts, even in a comment
 */
pub fn reads_time(shader: &Path) -> bool {
    READS_TIME.with_borrow_mut(|known| {
        if let Some(&reads) = known.get(shader) {
            return reads;
        }

        let source = fs::read_to_string(shader).unwrap_or_default();

        let is_name_part = |character: char| character.is_alphanumeric() || character == '_';

        let reads = source.split(|character| !is_name_part(character)).any(|word| word == "time");

        known.insert(shader.to_path_buf(), reads);

        reads
    })
}
