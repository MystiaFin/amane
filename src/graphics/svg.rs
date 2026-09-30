use std::any::TypeId;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock, Mutex};
use std::thread;

use vello::Scene;
use vello_svg::usvg::{Options, Tree};

use crate::services::wake;

// a vector picture, drawn fresh at whatever size it is shown, so it never blurs
pub struct Svg {
    pub(crate) scene: Scene,

    // the size the file says it is drawn at, which the scene is counted in
    width: f32,
    height: f32,
}

impl Svg {
    pub fn width(&self) -> f32 {
        self.width
    }

    pub fn height(&self) -> f32 {
        self.height
    }
}

// what a window waiting on a picture counts as having read, like images do
struct Parsed;

// every file asked for: none while it is still being read, or when it could not be
static LOADED: LazyLock<Mutex<HashMap<PathBuf, Option<Arc<Svg>>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

// whether amane draws this file as a vector picture instead of decoding its pixels
pub fn is_svg(path: &Path) -> bool {
    path.extension().is_some_and(|extension| extension.eq_ignore_ascii_case("svg"))
}

// read on its own thread like an image; none until it is ready, then the window is woken
pub fn load(path: &Path) -> Option<Arc<Svg>> {
    let mut loaded = LOADED.lock().expect("failed to lock loaded pictures");

    if let Some(svg) = loaded.get(path) {
        if svg.is_none() {
            wake::note_read(TypeId::of::<Parsed>());
        }

        return svg.clone();
    }

    loaded.insert(path.to_path_buf(), None);

    let path = path.to_path_buf();

    thread::spawn(move || parse(path));

    wake::note_read(TypeId::of::<Parsed>());

    None
}

// a file that can't be read stays empty
fn parse(path: PathBuf) {
    let Ok(bytes) = std::fs::read(&path) else {
        return;
    };

    let Ok(tree) = Tree::from_data(&bytes, &Options::default()) else {
        return;
    };

    let size = tree.size();

    let svg = Svg {
        scene: vello_svg::render_tree(&tree),
        width: size.width(),
        height: size.height(),
    };

    LOADED
        .lock()
        .expect("failed to lock loaded pictures")
        .insert(path, Some(Arc::new(svg)));

    wake::changed(TypeId::of::<Parsed>());
}
