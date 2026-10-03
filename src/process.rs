use std::io::{BufRead, BufReader};
use std::process::{Child, ChildStdout, Command, Stdio};
use std::thread;

pub struct Lines {
    child: Child,

    reader: BufReader<ChildStdout>,
}

impl Iterator for Lines {
    type Item = String;

    fn next(&mut self) -> Option<String> {
        let mut line = String::new();

        let read = self.reader.read_line(&mut line).ok()?;

        // 0 bytes means the command closed its output
        if read == 0 {
            return None;
        }

        let trimmed = line.trim_end_matches('\n');

        Some(String::from(trimmed))
    }
}

// a service that stops listening must not leave the command running
impl Drop for Lines {
    fn drop(&mut self) {
        let _ = self.child.kill();

        let _ = self.child.wait();
    }
}

// through the shell, so pipes, globs and ~ work like they do in a terminal
fn shell(command: &str) -> Command {
    let mut shell = Command::new("sh");

    shell.arg("-c").arg(command);

    shell
}

// starts a program and moves on without waiting for it
pub fn spawn(command: &str) {
    let mut child = shell(command).spawn().expect("failed to start command");

    // a finished child that nobody waits for stays behind as a zombie
    thread::spawn(move || child.wait());
}

// waits for the command to finish, so call it from a service thread, not from view()
pub fn output(command: &str) -> String {
    let output = shell(command).output().expect("failed to run command");

    let text = String::from_utf8_lossy(&output.stdout);

    // commands end their output with a newline that nobody wants on screen
    let trimmed = text.trim_end_matches('\n');

    String::from(trimmed)
}

// streams each line a long-running command prints, until it exits
pub fn lines(command: &str) -> Lines {
    let mut child = shell(command)
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to start command");

    let stdout = child.stdout.take().expect("failed to read command output");

    Lines {
        child,

        reader: BufReader::new(stdout),
    }
}
