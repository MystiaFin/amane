use smithay_client_toolkit::session_lock::{
    SessionLock, SessionLockHandler, SessionLockSurface, SessionLockSurfaceConfigure,
};
use wayland_client::{Connection, QueueHandle, protocol::wl_output::WlOutput};

use crate::{Lock, Service};

use super::{WaylandState, monitor, role::Role, settings::Settings, view::View};

impl WaylandState {
    // every monitor gets its own lock screen, the compositor shows nothing else while locked
    pub fn open_lock(&mut self, output: WlOutput) {
        let (Some(view), Some(session_lock)) = (self.lock_view, &self.session_lock) else {
            return;
        };

        // monitors described before the lock is granted get their screens in locked()
        if !session_lock.is_locked() {
            return;
        }

        let already_open = self.windows.iter().any(|window| {
            matches!(window.role, Role::Lock(_)) && window.output.as_ref() == Some(&output)
        });

        if already_open {
            return;
        }

        let Some(info) = self.output.info(&output) else {
            return;
        };

        let view = View::Monitor(view, monitor::describe(&info));

        let settings = Settings::from(&view.run());

        let surface = self.compositor.create_surface(&self.qh);

        let lock_surface = session_lock.create_lock_surface(surface, &output, &self.qh);

        self.add(view, Some(output), settings, Role::Lock(lock_surface));
    }

    // runs on every wake, since that is when a finished pam check shows up
    pub fn end_lock_if_unlocked(&mut self) {
        let Some(session_lock) = &self.session_lock else {
            return;
        };

        if !Lock::read().unlocked() {
            return;
        }

        session_lock.unlock();

        self.session_lock = None;

        self.close_lock_screens();

        /*
         * the unlock has to reach the compositor before amane exits,
         * or the session stays locked with nobody left to unlock it
         */
        self.connection
            .flush()
            .expect("failed to send unlock to the compositor");
    }

    fn close_lock_screens(&mut self) {
        self.windows
            .retain(|window| !matches!(window.role, Role::Lock(_)));

        if self.windows.is_empty() && self.per_monitor.is_empty() {
            self.running = false;
        }
    }
}

impl SessionLockHandler for WaylandState {
    fn locked(&mut self, _: &Connection, _: &QueueHandle<Self>, _: SessionLock) {
        let outputs: Vec<WlOutput> = self.output.outputs().collect();

        for output in outputs {
            self.open_lock(output);
        }
    }

    // the compositor refused, most likely because another locker already holds the session
    fn finished(&mut self, _: &Connection, _: &QueueHandle<Self>, _: SessionLock) {
        eprintln!("amane: the compositor refused or ended the session lock");

        self.session_lock = None;

        self.close_lock_screens();
    }

    fn configure(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        lock_surface: SessionLockSurface,
        configure: SessionLockSurfaceConfigure,
        _: u32,
    ) {
        let Some(window) = self.window(lock_surface.wl_surface()) else {
            return;
        };

        // a lock screen always covers its whole monitor
        let (width, height) = configure.new_size;

        window.width = width;
        window.height = height;

        window.redraw();
    }
}
