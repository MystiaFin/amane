use std::io::{BufRead, BufReader};
use std::process::{Child, ChildStdout, Stdio};

use super::shell;

pub struct Lines {
    child: Child,

    reader: BufReader<ChildStdout>,
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
