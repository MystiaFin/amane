// the numbers the spec gives each way a notification can close
#[derive(Clone, Copy)]
pub enum Reason {
    Dismissed = 2,

    // the sender asked with CloseNotification
    Closed = 3,
}
