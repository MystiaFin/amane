mod fallback;
mod family;
mod file;
mod measure;
mod sliders;

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

pub use family::{load_weighted, set_default};
pub use measure::{measure_letter, with_context};
pub use sliders::coords;

// a map shared by the whole program: things kept under a label, one user at a time
type SharedMap<Label, Thing> = LazyLock<Mutex<HashMap<Label, Thing>>>;

// every shared map starts out empty
fn empty_map<Label, Thing>() -> Mutex<HashMap<Label, Thing>> {
    Mutex::new(HashMap::new())
}
