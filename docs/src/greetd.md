# Login Greeters with greetd

`Greetd` connects to the socket in `GREETD_SOCK` and exposes greetd's authentication conversation as a Service. Use it when Amane itself is running as the greeter. It is separate from [`Lock`](lock_screen.md), which only unlocks an already running session.

```rust,ignore
use amane::{AuthenticationMessageType, Greetd, GreetdStatus, Service};

fn submit_username(username: &str) {
    Greetd::authenticate(username);
}

fn submit_prompt(value: &str) {
    Greetd::respond(value);
}

fn continue_after_information() {
    Greetd::acknowledge();
}

fn cancel() {
    Greetd::cancel();
}

fn start_sway() {
    Greetd::start_session(["sway"], ["XDG_SESSION_TYPE=wayland"]);
}
```

Read the Service in your view to choose what to show:

```rust,ignore
let greetd = Greetd::read();

if !greetd.connected() {
    // Show greetd.error(). Greetd::connect() retries the connection.
}

if let Some(prompt) = greetd.message() {
    match prompt.kind() {
        AuthenticationMessageType::Visible => { /* ordinary text field */ }
        AuthenticationMessageType::Secret => { /* hidden text field */ }
        AuthenticationMessageType::Information | AuthenticationMessageType::Error => {
            // Show prompt.text(), then call Greetd::acknowledge().
        }
    }
}

match greetd.status() {
    GreetdStatus::Authenticating => { /* waiting for or showing a prompt */ }
    GreetdStatus::Authenticated => { /* enable the session selector */ }
    GreetdStatus::AuthenticationFailed => { /* allow another attempt */ }
    GreetdStatus::StartingSession => { /* show progress */ }
    GreetdStatus::SessionStarted => std::process::exit(0),
    GreetdStatus::SessionFailed => { /* show greetd.error() */ }
    _ => {}
}
```

`message().text()` is the text supplied by PAM. Do not assume authentication only asks for one password: answer each `Visible` or `Secret` message separately, and acknowledge each `Information` or `Error` message. A successful authentication changes the status to `Authenticated`; it does not start anything until `start_session` is called.

All socket work happens on the Service thread, so authentication cannot freeze drawing. If greetd disconnects, the connection becomes `Disconnected`, the current operation returns to `Idle`, and `error()` describes the failure. Starting authentication again reconnects automatically, or call `Greetd::connect()` to retry immediately.
