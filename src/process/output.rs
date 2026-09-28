use super::shell;

// waits for the command to finish, so call it from a service thread, not from view()
pub fn output(command: &str) -> String {
    let output = shell(command).output().expect("failed to run command");

    let text = String::from_utf8_lossy(&output.stdout);

    // commands end their output with a newline that nobody wants on screen
    let trimmed = text.trim_end_matches('\n');

    String::from(trimmed)
}
