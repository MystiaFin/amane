# Polkit Authentication

`Polkit` lets your shell show the authentication dialogs requested by applications performing privileged actions. Polkit decides which actions need authentication and which accounts may authenticate them. Amane displays the request and handles the conversation through Polkit's native agent library and trusted helper.

## Registration

Reading the service does not register an agent. Start it once before running your app:

```rust,no_run
use amane::{Polkit, PolkitConfig};

Polkit::start(PolkitConfig::new())?;
# Ok::<(), amane::PolkitError>(())
```

Registration runs in the background. Read `running()` to see whether it succeeded and `error()` for failures. `enabled()` records whether registration has been requested. `Polkit::stop()` cancels pending requests and unregisters the agent; `start()` can register it again.

The process must belong to a login session known to logind, and Polkit's daemon and authentication helper must be installed. Only one agent can serve a session. If another agent is registered, Amane reports the registration error.

`PolkitConfig::new().path("/org/example/Agent").locale("en_US.UTF-8")` changes the agent's D-Bus object path and requested message locale. The defaults are `/org/amane/Polkit` and the first nonempty value of `LC_ALL`, `LC_MESSAGES`, or `LANG`, falling back to `C`. The login session is resolved from the process, rather than from an environment variable.

If Polkit or the system bus disconnects, the agent clears its requests, sets `running()` to false, and exposes the error. Call `start()` to retry registration.

## A request

`flow()` is `None` while idle. Otherwise it exposes:

| Function | Gives |
|---|---|
| `id()` | the request number used by control calls |
| `action_id()` | the action's machine-readable identifier |
| `message()` | the main message to show in the dialog |
| `icon()` | the requested icon name |
| `details()` | the action's additional string details |
| `identities()` | the permitted accounts, with `uid()` and `name()` |
| `selected_identity()` | the selected account's index |
| `prompt()` | the current input prompt, if one needs a response |
| `supplementary()` | the latest information or error message |
| `failed()` | whether an attempt has failed during this request |
| `can_retry()` | whether a rejected attempt is waiting for a retry |

Up to 32 additional requests wait in order. Completing or cancelling the active request opens the next one. A request is only accepted from the Polkit authority that registered the agent. Cancellation history is bounded; an overflow of unmatched cancellations disconnects the agent and reports an error rather than starting a dismissed request.

The native authentication session supports Unix user identities. Other identity types are rejected with an error. Account names fall back to numeric UIDs when they cannot be resolved.

## Prompts and responses

Show the prompt's `text()`. Use `visible()` to choose ordinary or masked input. Capture both the flow's `id()` and the prompt's `id()` when building an input handler:

```rust,ignore
let request = flow.id();
let prompt_id = prompt.id();

let mut input = TextInput::new("polkit-response")
    .focused()
    .on_submit(move |response| {
        if Polkit::respond(request, prompt_id, &response).is_ok() {
            TextInput::set_text("polkit-response", "");
        }
    });
if !prompt.visible() {
    input = input.password();
}
```

Each prompt gets a new ID, so a delayed response cannot answer a subsequent prompt or a different request. A conversation can ask for a password, an OTP, or other input; respond to the text and visibility of each prompt as it arrives. Responses containing NUL or line breaks are rejected by the helper's line protocol. Amane clears the response buffer it owns after passing it to the native library; clear your UI's input field too.

`supplementary()` has `text()` and `is_error()`. Display it independently of the input prompt.

## Identities, retries, and cancellation

Use `Polkit::select_identity(request_id, index)` to select one of the offered accounts. Changing the account cancels the previous conversation, clears its prompt and supplementary message, and starts a new conversation. Its old prompt IDs no longer work.

After a rejected attempt, `failed()` becomes true and the request stays open. When `can_retry()` is true, `Polkit::retry(request_id)` starts another attempt with the selected identity. The failure flag remains true for the rest of that request, even after a retry; `can_retry()` becomes false while the new attempt runs.

`Polkit::cancel(request_id)` dismisses the request. Polkit can also cancel it itself. `completion()` retains the most recently finished displayed request and its `PolkitResult`: `Success`, `Cancelled`, or `Error`. Cancelling a queued request does not replace that completion. This describes the authentication conversation; the final authorization decision remains with Polkit.

The control functions belong in startup or input handlers. A view reads the service and describes the dialog.

## Example

`cargo run --example polkit` starts an agent with a dialog for its requests. Run it as your shell's agent with other agents disabled. The example uses `App::without_ipc()` so it can run alongside your shell without claiming its IPC socket.

See the [Polkit agent documentation](https://polkit.pages.freedesktop.org/polkit/polkit.8.html) for how applications request authorization.
