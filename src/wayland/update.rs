use smithay_client_toolkit::shell::WaylandSurface;

use crate::LayerWindow;

use super::{layer, settings::Settings, window::Window};

impl Window {
    // settings changed by input or services reach the compositor before anything is drawn
    pub fn update_surface(&mut self, window: &LayerWindow) {
        let settings = Settings::from(window);

        if settings == self.settings {
            return;
        }

        let was_visible = self.settings.visible;

        self.settings = settings;

        // the rest waits until the window shows again, which sends every setting
        if !settings.visible {
            if was_visible {
                self.hide();
            }

            return;
        }

        /*
         * a commit without a buffer also shows a hidden window again,
         * the compositor answers with a configure and drawing starts there
         */
        layer::apply(&self.layer_surface, &settings);

        self.layer_surface.commit();
    }

    fn hide(&mut self) {
        // taking the buffer away unmaps the window and gives back its reserved space
        self.layer_surface.wl_surface().attach(None, 0, 0);

        self.layer_surface.commit();

        // nothing is drawn until showing the window brings a new configure
        self.width = 0;
        self.height = 0;

        // a frame callback asked for before hiding may never come
        self.frame_requested = false;
    }
}
