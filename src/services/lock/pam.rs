use std::ffi::{CString, c_char, c_int, c_void};
use std::ptr;

/*
 * linux-pam called straight through its c functions,
 * everything unsafe about the lock stays in this file
 */

// pam asks for the password with one of these, the rest are messages to show
const PROMPT_ECHO_OFF: c_int = 1;
const PROMPT_ECHO_ON: c_int = 2;

const SUCCESS: c_int = 0;
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

// pam calls this back whenever it needs an answer, the password rides along in data
type Converse =
    extern "C" fn(c_int, *mut *const Message, *mut *mut Response, *mut c_void) -> c_int;

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

    fn pam_authenticate(handle: *mut c_void, flags: c_int) -> c_int;

    // also refuses expired or locked accounts, which the password alone does not
    fn pam_acct_mgmt(handle: *mut c_void, flags: c_int) -> c_int;

    fn pam_end(handle: *mut c_void, status: c_int) -> c_int;
}

// pam frees the answers with free(), so they have to come from the c allocator
unsafe extern "C" {
    fn calloc(count: usize, size: usize) -> *mut c_void;

    fn strdup(text: *const c_char) -> *mut c_char;
}

// blocks for as long as pam takes, a wrong password usually waits a few seconds
pub fn authenticate(service: &str, user: &str, password: &str) -> bool {
    let (Ok(service), Ok(user), Ok(password)) = (
        CString::new(service),
        CString::new(user),
        CString::new(password),
    ) else {
        return false;
    };

    let conversation = Conversation {
        converse,
        data: password.as_ptr() as *mut c_void,
    };

    let mut handle = ptr::null_mut();

    // the password outlives the handle, pam_end runs before this returns
    unsafe {
        let started = pam_start(service.as_ptr(), user.as_ptr(), &conversation, &mut handle);

        if started != SUCCESS {
            return false;
        }

        let mut status = pam_authenticate(handle, 0);

        if status == SUCCESS {
            status = pam_acct_mgmt(handle, 0);
        }

        pam_end(handle, status);

        status == SUCCESS
    }
}

// every question pam asks gets the password, and every message gets an empty answer
extern "C" fn converse(
    count: c_int,
    messages: *mut *const Message,
    responses: *mut *mut Response,
    data: *mut c_void,
) -> c_int {
    let Ok(count) = usize::try_from(count) else {
        return CONVERSATION_ERROR;
    };

    let password = data as *const c_char;

    unsafe {
        let answers = calloc(count, size_of::<Response>()) as *mut Response;

        if answers.is_null() {
            return CONVERSATION_ERROR;
        }

        for index in 0..count {
            let message = &**messages.add(index);

            let asks = message.style == PROMPT_ECHO_OFF || message.style == PROMPT_ECHO_ON;

            if asks {
                (*answers.add(index)).text = strdup(password);
            }
        }

        *responses = answers;
    }

    SUCCESS
}
