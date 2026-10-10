# Lock Screen

Amane can lock your session with a lock screen you design yourself, and check the password through PAM, like your login screen does.

Your compositor must support the **ext-session-lock** protocol. niri, Hyprland, and Sway do.

## A lock screen

```rust
use amane::{
    App, Center, Color, Column, LayerWindow, Lock, Monitor, Parent, Rectangle, Service, Text,
    TextInput, children,
};

fn main() {
    App::new().lock(view).run();
}

fn view(_: &Monitor) -> LayerWindow {
    let lock = Lock::read();

    let status = if lock.checking() {
        "checking..."
    } else if lock.failed() {
        "wrong password"
    } else {
        "locked"
    };

    // the compositor sizes lock screens to the monitor, so this size is never used
    LayerWindow::new().width(1.0).height(1.0).child(
        Rectangle::new()
            .width(Parent)
            .height(Parent)
            .fill("#1e1e2e")
            .align_child(Center, Center)
            .child(
                Column::new(children![
                    Text::new(status).size(24.0).color(Color::WHITE),
                    Rectangle::new()
                        .width(300.0)
                        .height(40.0)
                        .fill(Color::WHITE)
                        .padding(8.0)
                        .child(
                            TextInput::new("password")
                                .placeholder("password")
                                .password()
                                .focused()
                                .on_submit(|password| Lock::unlock(&password)),
                        ),
                ])
                .gap(12.0),
            ),
    )
}
```

This sets up the lock screen, but doesn't lock anything yet. [Locking the session](#locking-the-session) shows how.

## How locking works

- `App::lock` takes a view that's shown on every monitor while the session is locked. Like `window_per_monitor`, it gets the `&Monitor` it's on.
- While the session is locked, the compositor shows only the lock screen, sends it all keys, and keeps every other window hidden. The lock screen's size, anchors, and layer are ignored. It always covers the whole monitor.
- `Lock::unlock(password)` checks the password through PAM (with the `login` service, as your user). That can take a few seconds, so it runs on its own thread. Authentication and account checks must both pass before the session unlocks. Otherwise, the screen stays locked.
- The `Lock` Service tells the view what's happening, through `checking()` and `failed()`.

## Locking the session

`Lock::start()` locks the session. Call it from anywhere: a button, an IPC handler, or a Service's thread. Calling it while already locked does nothing.

The usual setup is an IPC handler, so a keybind or an idle daemon can lock:

```rust,ignore
fn main() {
    App::new()
        .ipc("lock", |_| {
            Lock::start();

            String::from("locked")
        })
        .window(bar)
        .lock(lock_screen)
        .run();
}
```

Then lock from a keybind, or from swayidle or hypridle:

```sh
amane ipc call lock
```

For niri:

```kdl
binds {
    Mod+Alt+L { spawn "amane" "ipc" "call" "lock"; }
}
```

## Interactive authentication

For a configurable policy or a conversation with separate responses, start authentication from a handler:

```rust,ignore
Lock::authenticate(PamConfig::new("my-shell"))?;
```

Read the `Pam` Service in your lock view to show information, error messages and the current prompt. Answer each prompt with `Pam::respond(prompt.id(), response)`, hiding input when its kind is `PamMessageKind::Prompt { visible: false }`. See [PAM Authentication](pam.md) for a response handler and result types.

`Lock::authenticate` authenticates the current `$USER`, and rejects a configuration selecting another user. Both authentication and account checks must succeed for that same user to unlock. Standalone `Pam::start` does not unlock.

`Lock::abort()` cancels the lock's attempt and keeps the session locked. If a native module is still working, the PAM worker remains active until it returns; wait for `Pam::read().active()` to become false before retrying.

## Lock reference

| Function | Meaning |
|---|---|
| `Lock::read().checking()` | authentication is running |
| `Lock::read().failed()` | the last attempt was rejected or failed |
| `Lock::start()` | locks the session |
| `Lock::unlock(password)` | checks a password, and unlocks if it's right |
| `Lock::authenticate(config)` | starts interactive authentication, and unlocks on success |
| `Lock::abort()` | cancels authentication and keeps the session locked |

Only one PAM attempt runs at a time. Calling `unlock` while a check is running does nothing; `authenticate` returns `PamError::Busy`. The password convenience function supplies the same password to every prompt. Use interactive authentication when a policy asks for different responses. Each new lock starts clean, without the last lock's `failed` state.

## Testing safely

A locked session only opens with the right password. If your lock screen has a bug, like an input that doesn't take keys, you can't get back in from that session.

Killing the shell doesn't help either. If the program that locked the session exits without unlocking, the compositor keeps the session locked. That's on purpose: otherwise, crashing the lock screen would be a way past it.

Before using a new lock screen for real:

- Save your work in other apps.
- Switch to another TTY (`Ctrl+Alt+F3`), log in there, and keep it open. If you get stuck, switch to it and end the locked session with `loginctl terminate-session <id>` (`loginctl list-sessions` shows the id). This closes every app in that session.
- Try a wrong password first, then the right one.

## A blurred wallpaper background

```rust,ignore
Rectangle::new()
    .width(Parent)
    .height(Parent)
    .fill(Image::cover("/home/you/Pictures/wall.jpg").thumbnail(64, 36).blurred(4))
```

See [Images](images.md) for why the small thumbnail makes this cheap.
