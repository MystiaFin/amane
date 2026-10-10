use std::ffi::{CStr, CString, c_char, c_int, c_ulong, c_void};
use std::ptr::NonNull;
use std::sync::mpsc::Sender;

use super::{
    PolkitError,
    agent::{Command, NativeEvent},
};

pub(super) struct Secret(Vec<u8>);

impl Secret {
    pub fn new(value: &str) -> Result<Self, PolkitError> {
        if value.contains(['\0', '\r', '\n']) {
            return Err(PolkitError::InvalidResponse);
        }
        let mut bytes = value.as_bytes().to_vec();
        bytes.push(0);
        Ok(Self(bytes))
    }

    fn as_ptr(&self) -> *const c_char {
        self.0.as_ptr().cast()
    }
}

impl Drop for Secret {
    fn drop(&mut self) {
        for byte in &mut self.0 {
            // Volatile writes keep the owned response from surviving deallocation.
            unsafe {
                std::ptr::write_volatile(byte, 0);
            }
        }
    }
}

type Callback = unsafe extern "C" fn();

#[link(name = "polkit-agent-1")]
unsafe extern "C" {
    fn polkit_agent_session_new(identity: *mut c_void, cookie: *const c_char) -> *mut c_void;
    fn polkit_agent_session_initiate(session: *mut c_void);
    fn polkit_agent_session_response(session: *mut c_void, response: *const c_char);
    fn polkit_agent_session_cancel(session: *mut c_void);
}

#[link(name = "polkit-gobject-1")]
unsafe extern "C" {
    fn polkit_unix_user_new(uid: c_int) -> *mut c_void;
    fn polkit_unix_user_get_name(identity: *mut c_void) -> *const c_char;
}

#[link(name = "gobject-2.0")]
unsafe extern "C" {
    fn g_object_unref(object: *mut c_void);
    fn g_signal_connect_data(
        object: *mut c_void,
        signal: *const c_char,
        callback: Option<Callback>,
        data: *mut c_void,
        destroy: Option<unsafe extern "C" fn(*mut c_void, *mut c_void)>,
        flags: c_int,
    ) -> c_ulong;
    fn g_signal_handlers_disconnect_matched(
        object: *mut c_void,
        mask: c_int,
        id: u32,
        detail: u32,
        closure: *mut c_void,
        func: *mut c_void,
        data: *mut c_void,
    ) -> u32;
}

#[link(name = "glib-2.0")]
unsafe extern "C" {
    fn g_main_context_new() -> *mut c_void;
    fn g_main_context_push_thread_default(context: *mut c_void);
    fn g_main_context_pop_thread_default(context: *mut c_void);
    fn g_main_context_iteration(context: *mut c_void, may_block: c_int) -> c_int;
    fn g_main_context_unref(context: *mut c_void);
}

/// All native sessions and their callbacks stay on the service thread.
pub(super) struct Context(NonNull<c_void>);

impl Context {
    pub fn new() -> Result<Self, PolkitError> {
        let pointer = NonNull::new(unsafe { g_main_context_new() }).ok_or_else(|| {
            PolkitError::Authentication("cannot create the authentication event loop".into())
        })?;
        unsafe {
            g_main_context_push_thread_default(pointer.as_ptr());
        }
        Ok(Self(pointer))
    }
    pub fn dispatch(&self) {
        // Bound each batch so cancellation commands cannot be starved by native events.
        for _ in 0..32 {
            if unsafe { g_main_context_iteration(self.0.as_ptr(), 0) } == 0 {
                break;
            }
        }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            g_main_context_pop_thread_default(self.0.as_ptr());
            g_main_context_unref(self.0.as_ptr());
        }
    }
}

struct Events {
    sender: Sender<Command>,
    generation: u64,
}

impl Events {
    fn send(&self, event: NativeEvent) {
        let _ = self.sender.send(Command::Native(self.generation, event));
    }
}

pub(super) struct Session {
    pointer: NonNull<c_void>,
    events: Box<Events>,
}

impl Session {
    pub fn new(
        uid: u32,
        cookie: &str,
        generation: u64,
        sender: Sender<Command>,
    ) -> Result<Self, PolkitError> {
        let cookie = CString::new(cookie)
            .map_err(|_| PolkitError::Authentication("invalid authentication cookie".into()))?;
        let identity =
            NonNull::new(unsafe { polkit_unix_user_new(uid as c_int) }).ok_or_else(|| {
                PolkitError::Authentication("cannot create the authentication identity".into())
            })?;
        // The session retains its own reference to the identity and copies the cookie.
        let pointer = unsafe { polkit_agent_session_new(identity.as_ptr(), cookie.as_ptr()) };
        unsafe {
            g_object_unref(identity.as_ptr());
        }
        let pointer = NonNull::new(pointer).ok_or_else(|| {
            PolkitError::Authentication("cannot create the authentication session".into())
        })?;
        let mut session = Self {
            pointer,
            events: Box::new(Events { sender, generation }),
        };
        let data = (&mut *session.events as *mut Events).cast();
        // GObject's GCallback erases the signature; these match the documented session signals.
        unsafe {
            let request = std::mem::transmute::<
                unsafe extern "C" fn(*mut c_void, *const c_char, c_int, *mut c_void),
                Callback,
            >(request);
            let info = std::mem::transmute::<
                unsafe extern "C" fn(*mut c_void, *const c_char, *mut c_void),
                Callback,
            >(info);
            let error = std::mem::transmute::<
                unsafe extern "C" fn(*mut c_void, *const c_char, *mut c_void),
                Callback,
            >(error);
            let completed = std::mem::transmute::<
                unsafe extern "C" fn(*mut c_void, c_int, *mut c_void),
                Callback,
            >(completed);
            for (signal, callback) in [
                (c"request", request),
                (c"show-info", info),
                (c"show-error", error),
                (c"completed", completed),
            ] {
                g_signal_connect_data(
                    pointer.as_ptr(),
                    signal.as_ptr(),
                    Some(callback),
                    data,
                    None,
                    0,
                );
            }
        }
        Ok(session)
    }
    pub fn start(&self) {
        unsafe {
            polkit_agent_session_initiate(self.pointer.as_ptr());
        }
    }
    pub fn respond(&self, response: &Secret) {
        unsafe {
            polkit_agent_session_response(self.pointer.as_ptr(), response.as_ptr());
        }
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        // Detach callbacks before cancellation and before freeing their user data.
        unsafe {
            g_signal_handlers_disconnect_matched(
                self.pointer.as_ptr(),
                1 << 4,
                0,
                0,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                (&mut *self.events as *mut Events).cast(),
            );
            polkit_agent_session_cancel(self.pointer.as_ptr());
            g_object_unref(self.pointer.as_ptr());
        }
    }
}

pub(super) fn account_name(uid: u32) -> String {
    let Some(identity) = NonNull::new(unsafe { polkit_unix_user_new(uid as c_int) }) else {
        return uid.to_string();
    };
    let name = unsafe { polkit_unix_user_get_name(identity.as_ptr()) };
    let name = if name.is_null() {
        uid.to_string()
    } else {
        text(name)
    };
    unsafe {
        g_object_unref(identity.as_ptr());
    }
    name
}

fn text(value: *const c_char) -> String {
    if value.is_null() {
        return String::new();
    }
    // Signal strings are borrowed, NUL terminated, and valid until the callback returns.
    unsafe { CStr::from_ptr(value) }
        .to_string_lossy()
        .into_owned()
}

unsafe extern "C" fn request(
    _: *mut c_void,
    message: *const c_char,
    visible: c_int,
    data: *mut c_void,
) {
    unsafe { &*data.cast::<Events>() }.send(NativeEvent::Prompt(text(message), visible != 0));
}
unsafe extern "C" fn info(_: *mut c_void, message: *const c_char, data: *mut c_void) {
    unsafe { &*data.cast::<Events>() }.send(NativeEvent::Message(text(message), false));
}
unsafe extern "C" fn error(_: *mut c_void, message: *const c_char, data: *mut c_void) {
    unsafe { &*data.cast::<Events>() }.send(NativeEvent::Message(text(message), true));
}
unsafe extern "C" fn completed(_: *mut c_void, success: c_int, data: *mut c_void) {
    unsafe { &*data.cast::<Events>() }.send(NativeEvent::Completed(success != 0));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;

    unsafe extern "C" {
        fn g_signal_emit_by_name(object: *mut c_void, signal: *const c_char, ...);
    }

    #[test]
    fn native_session_callbacks_preserve_prompt_types_messages_and_completion() {
        let _context = Context::new().unwrap();
        let (sender, receiver) = mpsc::channel();
        let session = Session::new(0, "test-cookie", 17, sender).unwrap();
        // Exercise the real native GObject and FFI callbacks without invoking the privileged helper.
        unsafe {
            g_signal_emit_by_name(
                session.pointer.as_ptr(),
                c"request".as_ptr(),
                c"Password:".as_ptr(),
                0 as c_int,
            );
            g_signal_emit_by_name(
                session.pointer.as_ptr(),
                c"request".as_ptr(),
                c"One-time code:".as_ptr(),
                1 as c_int,
            );
            g_signal_emit_by_name(
                session.pointer.as_ptr(),
                c"show-info".as_ptr(),
                c"Use your account password".as_ptr(),
            );
            g_signal_emit_by_name(
                session.pointer.as_ptr(),
                c"show-error".as_ptr(),
                c"Incorrect password".as_ptr(),
            );
            g_signal_emit_by_name(session.pointer.as_ptr(), c"completed".as_ptr(), 1 as c_int);
        }
        assert!(
            matches!(receiver.try_recv().unwrap(), Command::Native(17, NativeEvent::Prompt(text, false)) if text == "Password:")
        );
        assert!(
            matches!(receiver.try_recv().unwrap(), Command::Native(17, NativeEvent::Prompt(text, true)) if text == "One-time code:")
        );
        assert!(
            matches!(receiver.try_recv().unwrap(), Command::Native(17, NativeEvent::Message(text, false)) if text == "Use your account password")
        );
        assert!(
            matches!(receiver.try_recv().unwrap(), Command::Native(17, NativeEvent::Message(text, true)) if text == "Incorrect password")
        );
        assert!(matches!(
            receiver.try_recv().unwrap(),
            Command::Native(17, NativeEvent::Completed(true))
        ));
        drop(session);
        assert!(receiver.try_recv().is_err());
    }
}
