use std::io::{self, Read};
use std::net::Shutdown;
use std::os::unix::net::UnixStream;
use std::process::ExitCode;

use amane::{IpcCall, ipc_socket};

pub fn run(name: &str, arguments: &[String]) -> ExitCode {
    // a newline would split one argument into two on the shell's side
    let has_newline = arguments.iter().any(|argument| argument.contains('\n'));

    if name.contains('\n') || has_newline {
        eprintln!("failed to call: names and arguments can't contain newlines");

        return ExitCode::FAILURE;
    }

    let Ok(stream) = UnixStream::connect(ipc_socket()) else {
        eprintln!("failed to call: amane is not running");

        return ExitCode::FAILURE;
    };

    let call = IpcCall::new(name, arguments);

    let Ok(reply) = send(stream, &call) else {
        eprintln!("failed to call: the shell closed the connection");

        return ExitCode::FAILURE;
    };

    println!("{}", reply.trim_end());

    ExitCode::SUCCESS
}

fn send(mut stream: UnixStream, call: &IpcCall) -> io::Result<String> {
    call.write(&mut stream)?;

    // closing our sending side is how the shell knows the call is complete
    stream.shutdown(Shutdown::Write)?;

    let mut reply = String::new();

    stream.read_to_string(&mut reply)?;

    Ok(reply)
}
