use std::fs;
use std::path::Path;

include!(concat!(env!("OUT_DIR"), "/library.rs"));

/*
 * only files whose contents changed are rewritten,
 * because a newer timestamp makes cargo rebuild amane from scratch
 */
pub fn unpack(folder: &Path) {
    for (name, contents) in FILES {
        let path = folder.join(name);

        if fs::read(&path).is_ok_and(|current| current == *contents) {
            continue;
        }

        let parent = path.parent().expect("failed to find library folder");

        fs::create_dir_all(parent).expect("failed to create library folder");

        fs::write(&path, contents).expect("failed to unpack library");
    }
}
