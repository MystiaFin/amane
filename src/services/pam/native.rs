use std::ffi::{CStr, CString, c_char, c_int, c_void};
use std::panic::{self, AssertUnwindSafe};
use std::ptr;
use std::sync::Arc;

use super::config::PreparedConfig;
use super::{PamError, PamMessageKind, PamResult, Session};

const SUCCESS: c_int = 0;
const BUFFER_ERROR: c_int = 5;
const CONVERSATION_ERROR: c_int = 19;

#[repr(C)]
struct Message {
    style: c_int,
    text: *const c_char,
}

#[repr(C)]
struct Response {
    text: *mut c_char,
    code: c_int,
}

type Converse =
    unsafe extern "C" fn(c_int, *mut *const Message, *mut *mut Response, *mut c_void) -> c_int;

#[repr(C)]
struct Conversation {
    converse: Converse,
    data: *mut c_void,
}

#[link(name = "pam")]
unsafe extern "C" {
    fn pam_start(
        service: *const c_char,
        user: *const c_char,
        conversation: *const Conversation,
        handle: *mut *mut c_void,
    ) -> c_int;
    fn pam_start_confdir(
        service: *const c_char,
        user: *const c_char,
        conversation: *const Conversation,
        directory: *const c_char,
        handle: *mut *mut c_void,
    ) -> c_int;
    fn pam_authenticate(handle: *mut c_void, flags: c_int) -> c_int;
    fn pam_acct_mgmt(handle: *mut c_void, flags: c_int) -> c_int;
    fn pam_get_item(handle: *const c_void, item: c_int, value: *mut *const c_void) -> c_int;
    fn pam_strerror(handle: *mut c_void, code: c_int) -> *const c_char;
    fn pam_end(handle: *mut c_void, status: c_int) -> c_int;
    fn calloc(count: usize, size: usize) -> *mut c_void;
    fn strdup(text: *const c_char) -> *mut c_char;
    fn free(pointer: *mut c_void);
}

struct Handle {
    pointer: *mut c_void,
    status: c_int,
}

impl Drop for Handle {
    fn drop(&mut self) {
        // only a successfully started handle is owned, and its conversation is still alive
        unsafe {
            pam_end(self.pointer, self.status);
        }
    }
}

struct Context<'a> {
    session: &'a Arc<Session>,
    password: Option<&'a str>,
}

pub(super) fn authenticate(
    config: &PreparedConfig,
    session: &Arc<Session>,
    password: Option<&str>,
) -> PamResult {
    let mut context = Context { session, password };
    let conversation = Conversation {
        converse,
        data: ptr::from_mut(&mut context).cast(),
    };
    let mut pointer = ptr::null_mut();
    // all strings and the callback context outlive the handle; PAM stays on this worker thread
    let status = unsafe {
        match &config.directory {
            Some(directory) => pam_start_confdir(
                config.service.as_ptr(),
                config.user.as_ptr(),
                &conversation,
                directory.as_ptr(),
                &mut pointer,
            ),
            None => pam_start(
                config.service.as_ptr(),
                config.user.as_ptr(),
                &conversation,
                &mut pointer,
            ),
        }
    };
    if status != SUCCESS {
        return classify(ptr::null_mut(), status);
    }
    let mut handle = Handle { pointer, status };
    if session.aborted() {
        return PamResult::Aborted;
    }
    handle.status = unsafe { pam_authenticate(handle.pointer, 0) };
    if session.aborted() {
        return PamResult::Aborted;
    }
    if handle.status == SUCCESS {
        // accepting a password must not bypass account expiry or access restrictions
        handle.status = unsafe { pam_acct_mgmt(handle.pointer, 0) };
    }
    if session.aborted() {
        return PamResult::Aborted;
    }
    if handle.status != SUCCESS {
        return classify(handle.pointer, handle.status);
    }

    let mut user = ptr::null();
    handle.status = unsafe { pam_get_item(handle.pointer, 2, &mut user) }; // PAM_USER
    if handle.status != SUCCESS {
        return classify(handle.pointer, handle.status);
    }
    // a module can change PAM_USER; success for someone else must not unlock this user's session
    if user.is_null() || unsafe { CStr::from_ptr(user.cast()) } != config.user.as_c_str() {
        handle.status = 7; // PAM_AUTH_ERR
        return PamResult::Error(PamError::UserChanged);
    }
    PamResult::Success
}

fn classify(handle: *mut c_void, code: c_int) -> PamResult {
    let text = unsafe { pam_strerror(handle, code) };
    let message = if text.is_null() {
        String::from("unknown error")
    } else {
        unsafe { CStr::from_ptr(text) }
            .to_string_lossy()
            .into_owned()
    };
    let error = PamError::Native { code, message };
    match code {
        // policy or credential rejection, distinct from a broken/unavailable PAM backend
        6 | 7 | 10 | 11 | 12 | 13 => PamResult::Rejected(error),
        _ => PamResult::Error(error),
    }
}

// responses use the C allocator because the PAM module frees them after a successful callback
struct Answers {
    pointer: *mut Response,
    count: usize,
}

impl Answers {
    fn release(mut self) -> *mut Response {
        std::mem::replace(&mut self.pointer, ptr::null_mut())
    }
}

impl Drop for Answers {
    fn drop(&mut self) {
        if self.pointer.is_null() {
            return;
        }
        unsafe {
            for index in 0..self.count {
                free((*self.pointer.add(index)).text.cast());
            }
            free(self.pointer.cast());
        }
    }
}

unsafe extern "C" fn converse(
    count: c_int,
    messages: *mut *const Message,
    responses: *mut *mut Response,
    data: *mut c_void,
) -> c_int {
    // a Rust panic must never cross PAM's C stack; partial response allocations have their own guard
    panic::catch_unwind(AssertUnwindSafe(|| unsafe {
        answer(count, messages, responses, data)
    }))
    .unwrap_or(CONVERSATION_ERROR)
}

unsafe fn answer(
    count: c_int,
    messages: *mut *const Message,
    responses: *mut *mut Response,
    data: *mut c_void,
) -> c_int {
    if !(1..=32).contains(&count) || messages.is_null() || responses.is_null() || data.is_null() {
        return CONVERSATION_ERROR;
    }
    let count = count as usize;
    let pointer = unsafe { calloc(count, size_of::<Response>()) }.cast::<Response>();
    if pointer.is_null() {
        return BUFFER_ERROR;
    }
    let answers = Answers { pointer, count };
    let context = unsafe { &*data.cast::<Context<'_>>() };
    for index in 0..count {
        // Linux-PAM supplies an array of message pointers, not a contiguous Message array
        let pointer = unsafe { *messages.add(index) };
        if pointer.is_null() {
            return CONVERSATION_ERROR;
        }
        let message = unsafe { &*pointer };
        let kind = match message.style {
            1 => PamMessageKind::Prompt { visible: false },
            2 => PamMessageKind::Prompt { visible: true },
            3 => PamMessageKind::Error,
            4 => PamMessageKind::Info,
            _ => return CONVERSATION_ERROR,
        };
        if message.text.is_null() {
            return CONVERSATION_ERROR;
        }
        let text = unsafe { CStr::from_ptr(message.text) }
            .to_string_lossy()
            .into_owned();
        let Ok(response) = super::message(context.session, text, kind, context.password) else {
            return CONVERSATION_ERROR;
        };
        if let Some(response) = response {
            let Ok(response) = CString::new(response) else {
                return CONVERSATION_ERROR;
            };
            let text = unsafe { strdup(response.as_ptr()) };
            if text.is_null() {
                return BUFFER_ERROR;
            }
            unsafe {
                (*answers.pointer.add(index)).text = text;
            }
        }
    }
    unsafe {
        *responses = answers.release();
    }
    SUCCESS
}
