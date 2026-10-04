mod bar;

use amane::*;

fn main() {
    App::new().window_per_monitor(bar::view).run();
}
