use std::env;
use std::io::Write;
use std::os::unix::net::UnixStream;

// niri reads one json request per line, then answers on the same connection
pub fn send(request: &str) -> UnixStream {
    let path = env::var("NIRI_SOCKET").expect("failed to find niri: NIRI_SOCKET is not set");

    let mut stream = UnixStream::connect(path).expect("failed to connect to niri");

    let line = format!("{request}\n");

    stream
        .write_all(line.as_bytes())
        .expect("failed to send request to niri");

    stream
}
