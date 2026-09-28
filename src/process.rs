mod lines;
mod output;
mod spawn;

use std::process::Command;

pub use lines::lines;
pub use output::output;
pub use spawn::spawn;

// through the shell, so pipes, globs and ~ work like they do in a terminal
fn shell(command: &str) -> Command {
    let mut shell = Command::new("sh");

    shell.arg("-c").arg(command);

    shell
}
