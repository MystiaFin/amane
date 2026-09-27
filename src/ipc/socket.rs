use std::env;
use std::path::PathBuf;

pub fn ipc_socket() -> PathBuf {
    // every wayland session sets this, and it is private to the logged in user
    let runtime_directory =
        env::var_os("XDG_RUNTIME_DIR").expect("failed to find socket: XDG_RUNTIME_DIR is not set");

    PathBuf::from(runtime_directory).join("amane.sock")
}
