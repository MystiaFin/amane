use std::any::{self, TypeId};
use std::ops::{Deref, DerefMut};
use std::sync::RwLockWriteGuard;

use crate::changes;
use crate::timing;

pub struct Write<S: 'static> {
    pub(crate) guard: RwLockWriteGuard<'static, S>,

    // set when the write turned out to change nothing a window shows
    pub(crate) quiet: bool,
}

impl<S> Write<S> {
    // finishes without waking any window, for a poll that found nothing new
    pub(crate) fn quiet(&mut self) {
        self.quiet = true;
    }
}

impl<S> Deref for Write<S> {
    type Target = S;

    fn deref(&self) -> &S {
        &self.guard
    }
}

impl<S> DerefMut for Write<S> {
    fn deref_mut(&mut self) -> &mut S {
        &mut self.guard
    }
}

// a finished write is a change the windows that read the service have to show
impl<S: 'static> Drop for Write<S> {
    fn drop(&mut self) {
        if self.quiet {
            return;
        }

        // with AMANE_FRAMES, the frame log also says which service woke the windows
        if timing::enabled() {
            eprintln!("change {}", any::type_name::<S>());
        }

        changes::mark(TypeId::of::<S>());
    }
}
