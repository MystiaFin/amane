# PAM Authentication

The `Pam` Service runs a PAM conversation on a background thread. It exposes each prompt and message so your shell can ask for a password, a verification code, or another response. A successful standalone `Pam` attempt does not unlock the session. Use `Lock::authenticate` for [interactive unlocking](lock_screen.md#interactive-authentication).

## Starting authentication

Call `start` from an input handler or another thread:

```rust,ignore
use amane::{Pam, PamConfig};

Pam::start(PamConfig::new("login"))?;
```

The service name selects a policy file in `/etc/pam.d`. Choose a policy appropriate for your shell. The user defaults to `$USER`; standalone authentication can choose another user:

```rust,ignore
Pam::start(PamConfig::new("my-shell").user("alice"))?;
```

`PamConfig::config_directory(path)` selects a different policy directory. The service name is still a filename within that directory. This uses Linux-PAM's `pam_start_confdir` and is useful for application policies and isolated tests.

Only one attempt runs at a time, including attempts started by `Lock`. `start` returns `PamError::Busy` if another worker is still active. Starting a new attempt clears the previous messages and result.

## Prompts and messages

Read `Pam` in a view, as you would any other [Service](services.md). Each `PamMessage` has an `id()`, `text()`, and `kind()`:

| Kind | Meaning |
|---|---|
| `PamMessageKind::Info` | information to show the user |
| `PamMessageKind::Error` | a message from the PAM module to show the user |
| `PamMessageKind::Prompt { visible: true }` | a prompt whose response can be shown while typing |
| `PamMessageKind::Prompt { visible: false }` | a prompt whose response should be hidden |

`messages()` keeps the conversation's message history. `prompt()` returns only the current unanswered prompt. Prompts are answered one at a time, even when a module sends several together.

Capture the prompt's ID in the input handler. This prevents a delayed submission from answering a later prompt or another attempt:

```rust,ignore
use amane::{Pam, PamMessageKind, Service, TextInput};

let pam = Pam::read();
if let Some(prompt) = pam.prompt() {
    let id = prompt.id();
    let mut input = TextInput::new("pam-response")
        .placeholder(prompt.text())
        .focused()
        .on_submit(move |response| {
            if Pam::respond(id, &response).is_ok() {
                TextInput::set_text("pam-response", "");
            }
        });
    if prompt.kind() == (PamMessageKind::Prompt { visible: false }) {
        input = input.password();
    }
    // Add input to the view's widget tree.
}
```

Responses are accepted once. A stale ID or duplicate response returns `PamError::NoPrompt`. A response containing a NUL byte is rejected without consuming the prompt. Responses are not added to message history.

Views only read Services. Call `start`, `respond`, and `abort` from handlers, after any `Pam::read()` or `Lock::read()` guard has been released.

## Results and cancellation

`result()` is `None` while an attempt is running, unless it has been cancelled. It then contains one of these results:

| Result | Meaning |
|---|---|
| `PamResult::Success` | authentication and account checks both passed |
| `PamResult::Rejected(error)` | credentials or account policy rejected the attempt |
| `PamResult::Aborted` | the attempt was cancelled |
| `PamResult::Error(error)` | the PAM backend or worker failed |

`PamError::Native { code, message }` preserves PAM's error code and text. A module changing the authenticated user produces `PamError::UserChanged`. Displaying a module's `Error` message alone does not mean the attempt has finished; use `result()` for its final outcome.

`Pam::abort()` immediately cancels the attempt and wakes a waiting prompt. A native module already doing work outside the conversation callback must return before it can be cleaned up. `active()` stays true until that cleanup finishes, and another start returns `Busy` until then. A cancelled attempt's late result is discarded.

For a lock-owned attempt, `Lock::abort()` also clears the lock's checking and failure state and keeps the session locked. A new session lock cancels its old authentication attempt.

## Example

`cargo run --example pam` opens a normal authentication window. It uses the `login` policy and does not lock or unlock your session. The example uses `App::without_ipc()` so it can run alongside your shell without claiming its IPC socket. You can select a policy and directory with `AMANE_PAM_SERVICE` and `AMANE_PAM_CONFIG_DIRECTORY`.

The library tests compile a temporary PAM module and use a temporary policy directory. They exercise conversations, account rejection, cancellation and stale results without checking real credentials or changing system authentication settings.
