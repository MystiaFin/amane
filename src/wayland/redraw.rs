use std::time::Instant;

use smithay_client_toolkit::compositor::FrameCallbackData;

use crate::frame;
use crate::graphics::Renderer;
use crate::timing::{self, Timing};

use super::surface::OpenWindow;

impl OpenWindow {
    // one frame: run the view, send its settings, draw it and show it
    pub fn redraw(&mut self) {
        let started = Instant::now();

        let (content, reads) = frame::run_view(
            || self.view.run(),
            self.width,
            self.height,
            self.scale_factor,
        );

        // a later change to one of these services draws this window again
        self.reads = reads;

        let viewed = Instant::now();

        let name = content.name();

        self.update_surface(&content);

        // 0 until the compositor configures the window, and again while it is hidden
        if self.width == 0 || self.height == 0 {
            return;
        }

        let (root, on_key) = content.into_parts();

        // a window without a child has nothing to show, which is a mistake but not a crash
        let Some(root) = root else {
            eprintln!("amane: {name} has no child to draw, see child()");

            return;
        };

        let frame = frame::build(
            root.as_ref(),
            self.width,
            self.height,
            self.scale,
            self.scale_factor,
        );

        // some widgets read services while drawing
        self.reads.extend(frame.reads);

        // the handlers are rebuilt with the view, so each frame replaces the last frame's
        self.pointer.set_targets(frame.targets);

        self.on_key = on_key;

        /*
         * an animation that has not arrived yet needs the next frame too; asking
         * in the same commit as this frame gets the answer on the next refresh,
         * a hidden window never gets here so it waits until it shows again
         */
        if frame.moving {
            self.ask_for_frame();
        }

        let drawn = Instant::now();

        let presented = self.present(frame.renderer);

        let timing = Timing {
            started,
            viewed,
            drawn,
            presented: Instant::now(),
        };

        timing::log(name, self.width, self.height, self.last_frame, &timing);

        self.last_frame = Some(started);

        // a skipped frame commits nothing, so the request goes out on its own
        if !presented && frame.moving {
            self.role.commit();
        }
    }

    pub fn request_frame(&mut self) {
        /*
         * a hidden or unconfigured window gets no frame callbacks,
         * so the view runs right away to see if the window should show
         */
        if self.width == 0 {
            self.redraw();

            return;
        }

        // changes that land before the next frame all draw together in it
        if self.frame_requested {
            return;
        }

        self.ask_for_frame();

        self.role.commit();
    }

    // the compositor answers once, when the next frame is due
    fn ask_for_frame(&mut self) {
        if self.frame_requested {
            return;
        }

        self.frame_requested = true;

        let surface = self.role.wl_surface();

        surface.frame(&self.qh, FrameCallbackData(surface.clone()));
    }

    /*
     * false when the gpu skipped the frame, so nothing was committed; the window
     * is measured in logical pixels, the buffer in real ones, and the scale goes
     * out with the commit that presenting the frame makes
     */
    fn present(&mut self, renderer: Renderer) -> bool {
        let (buffer_width, buffer_height) = match &self.fractional {
            // a buffer at the exact scale, shown back at the window's size
            Some(fractional) => {
                let width = self.width as i32;
                let height = self.height as i32;

                fractional.viewport.set_destination(width, height);

                (
                    to_real_pixels(self.width, self.scale),
                    to_real_pixels(self.height, self.scale),
                )
            }

            None => {
                self.role.wl_surface().set_buffer_scale(self.scale as i32);

                (
                    self.width * self.scale as u32,
                    self.height * self.scale as u32,
                )
            }
        };

        self.gpu
            .draw(renderer.finish(), buffer_width, buffer_height)
    }
}

// a logical size in real pixels at a fractional scale
fn to_real_pixels(size: u32, scale: f32) -> u32 {
    let pixels = size as f32 * scale;

    pixels.round() as u32
}
