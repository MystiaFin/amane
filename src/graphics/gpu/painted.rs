use vello::Scene;
use vello::wgpu::Texture;

use super::Gpu;

// what vello drew at one place in the frame, kept with the scene that drew it
pub struct Painted {
    scene: Scene,
    result: Texture,
}

/*
 * vello costs about the same however little it draws, and most frames
 * draw the same shapes as the frame before, like a ring that didn't move,
 * so an unchanged scene lays last frame's result again instead
 */
impl Gpu {
    // last frame's result for the scene painted at this place, if nothing changed
    pub(super) fn unchanged(&self, index: usize, scene: &Scene, canvas: &Texture) -> Option<&Texture> {
        let Some(Some(painted)) = self.painted.get(index) else {
            return None;
        };

        let same_size =
            painted.result.width() == canvas.width() && painted.result.height() == canvas.height();

        if !same_size || !same(&painted.scene, scene) {
            return None;
        }

        Some(&painted.result)
    }

    // scenes with images or gradients point at textures that can change, so those are never kept
    pub(super) fn keep(&mut self, index: usize, scene: &Scene, result: Texture) {
        if self.painted.len() <= index {
            self.painted.resize_with(index + 1, || None);
        }

        let resources = &scene.encoding().resources;

        let painted = if resources.patches.is_empty() && resources.glyph_runs.is_empty() {
            Some(Painted {
                scene: scene.clone(),
                result,
            })
        } else {
            self.give_back(result);

            None
        };

        let old = std::mem::replace(&mut self.painted[index], painted);

        if let Some(old) = old {
            self.give_back(old.result);
        }
    }

    // places this frame didn't reach are gone, like a closed popup
    pub(super) fn forget_unpainted(&mut self) {
        while self.painted.len() > self.painting {
            let Some(Some(old)) = self.painted.pop() else {
                continue;
            };

            self.give_back(old.result);
        }
    }
}

fn same(kept: &Scene, scene: &Scene) -> bool {
    let kept = kept.encoding();
    let scene = scene.encoding();

    kept.path_tags == scene.path_tags
        && kept.path_data == scene.path_data
        && kept.draw_tags == scene.draw_tags
        && kept.draw_data == scene.draw_data
        && kept.transforms == scene.transforms
        && kept.styles == scene.styles
        && kept.flags == scene.flags
}
