use crate::graphics::Rect;

use super::Target;

/*
 * a target that is clipped only reacts where it still shows,
 * the ones fully clipped out stay in the list so positions don't shift
 */
pub fn clip(targets: &mut [Target], area: Rect) {
    for target in targets {
        target.area = target.area.intersect(area);
    }
}
