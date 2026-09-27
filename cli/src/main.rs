mod call;

use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
    let arguments: Vec<String> = env::args().skip(1).collect();

    match arguments.as_slice() {
        [group, action, name, rest @ ..] if group == "ipc" && action == "call" => {
            call::run(name, rest)
        }

        _ => usage(),
    }
}

fn usage() -> ExitCode {
    eprintln!("usage: amane ipc call <name> [arguments...]");

    ExitCode::FAILURE
}
