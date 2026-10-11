use std::io::{self, Read, Write};
use std::os::unix::net::UnixStream;

use serde_json::{Value, json};

const MAX_REPLY_SIZE: usize = 1024 * 1024;

pub enum Reply {
    Success,
    Message { kind: String, text: String },
    Error { kind: String, description: String },
}

pub fn create_session(stream: &mut UnixStream, username: &str) -> io::Result<Reply> {
    request(
        stream,
        json!({ "type": "create_session", "username": username }),
    )
}

pub fn respond(stream: &mut UnixStream, response: Option<&str>) -> io::Result<Reply> {
    let message = match response {
        Some(response) => json!({
            "type": "post_auth_message_response",
            "response": response,
        }),
        None => json!({ "type": "post_auth_message_response" }),
    };

    request(stream, message)
}

pub fn cancel(stream: &mut UnixStream) -> io::Result<Reply> {
    request(stream, json!({ "type": "cancel_session" }))
}

pub fn start_session(
    stream: &mut UnixStream,
    command: &[String],
    environment: &[String],
) -> io::Result<Reply> {
    request(
        stream,
        json!({ "type": "start_session", "cmd": command, "env": environment }),
    )
}

fn request(stream: &mut UnixStream, request: Value) -> io::Result<Reply> {
    let payload = serde_json::to_vec(&request).map_err(io::Error::other)?;
    let length = u32::try_from(payload.len())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "greetd request is too large"))?;

    stream.write_all(&length.to_ne_bytes())?;
    stream.write_all(&payload)?;

    let mut length = [0; 4];
    stream.read_exact(&mut length)?;
    let length = u32::from_ne_bytes(length) as usize;

    if length > MAX_REPLY_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "greetd reply is too large",
        ));
    }

    let mut payload = vec![0; length];
    stream.read_exact(&mut payload)?;

    let reply: Value = serde_json::from_slice(&payload).map_err(io::Error::other)?;
    let kind = reply["type"]
        .as_str()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "greetd reply has no type"))?;

    match kind {
        "success" => Ok(Reply::Success),
        "auth_message" => Ok(Reply::Message {
            kind: required_text(&reply, "auth_message_type")?,
            text: required_text(&reply, "auth_message")?,
        }),
        "error" => Ok(Reply::Error {
            kind: required_text(&reply, "error_type")?,
            description: required_text(&reply, "description")?,
        }),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("unknown greetd reply type: {kind}"),
        )),
    }
}

fn required_text(value: &Value, field: &str) -> io::Result<String> {
    value[field].as_str().map(String::from).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("greetd reply has no {field}"),
        )
    })
}

#[cfg(test)]
mod tests {
    use std::os::unix::net::UnixStream;
    use std::thread;

    use super::*;

    #[test]
    fn sends_and_reads_framed_json() {
        let (mut client, mut server) = UnixStream::pair().unwrap();

        let handle = thread::spawn(move || {
            let mut length = [0; 4];
            server.read_exact(&mut length).unwrap();

            let mut request = vec![0; u32::from_ne_bytes(length) as usize];
            server.read_exact(&mut request).unwrap();

            let request: Value = serde_json::from_slice(&request).unwrap();
            assert_eq!(request["type"], "create_session");
            assert_eq!(request["username"], "mystia");

            let reply = br#"{"type":"auth_message","auth_message_type":"secret","auth_message":"Password:"}"#;
            server
                .write_all(&(reply.len() as u32).to_ne_bytes())
                .unwrap();
            server.write_all(reply).unwrap();
        });

        let Reply::Message { kind, text } = create_session(&mut client, "mystia").unwrap() else {
            panic!("expected an authentication message");
        };

        assert_eq!(kind, "secret");
        assert_eq!(text, "Password:");
        handle.join().unwrap();
    }
}
