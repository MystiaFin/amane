use std::any::TypeId;
use std::ops::{Deref, DerefMut};
use std::sync::RwLockWriteGuard;

use super::wake;

pub struct Write<S: 'static> {
    pub(crate) guard: RwLockWriteGuard<'static, S>,
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
        wake::changed(TypeId::of::<S>());
    }
}
