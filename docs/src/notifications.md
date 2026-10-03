# Notifications

`Notifications` makes your shell the notification daemon. Programs send their notifications to Amane, the same way they'd send them to mako or dunst, and you decide how to show them.

## Before you start

Only one notification daemon can run at a time. Stop the one you have (mako, dunst, swaync, or a notification feature in another bar) before starting your shell. If another daemon is running, `running()` returns `false` and Amane receives nothing.

Amane starts being the daemon the first time something reads `Notifications`, so read it in a view that's open when the shell starts.

## A notification list

```rust
use amane::{
    App, Column, Horizontal, Layer, LayerWindow, Notification, Notifications, Parent, Rectangle,
    Service, Text, Urgency, Vertical, Widget,
};

fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    let notifications = Notifications::read();

    let mut cards: Vec<Box<dyn Widget>> = Vec::new();

    for notification in notifications.list() {
        cards.push(Box::new(card(notification)));
    }

    LayerWindow::new()
        .width(400.0)
        .height(600.0)
        .anchor_vertical(Vertical::Top)
        .anchor_horizontal(Horizontal::Right)
        .layer(Layer::Overlay)
        .visible(!notifications.list().is_empty())
        .child(Column::new(cards).gap(8.0))
}

// click a notification to run its default action, or to dismiss it
fn card(notification: &Notification) -> Rectangle {
    let fill = match notification.urgency() {
        Urgency::Critical => "#f38ba8",
        _ => "#313244",
    };

    let id = notification.id();

    let label = format!("{}: {}", notification.summary(), notification.body());

    Rectangle::new()
        .width(Parent)
        .height(60.0)
        .radius(12.0)
        .fill(fill)
        .padding(12.0)
        .on_click(move |_| Notifications::click(id))
        .child(Text::new(label).elide())
}
```

The window hides itself while there are no notifications, so its empty space doesn't block clicks on the windows underneath.

Try it with:

```sh
notify-send "hello" "from amane"
```

## Reading notifications

`list()` gives every notification, oldest first. Each `Notification` has:

| Function | Gives |
|---|---|
| `id()` | its number, which the control functions take |
| `app_name()` | the sending program's name |
| `summary()` | the one-line title |
| `body()` | the text, which may be empty and may hold simple markup like `<b>` |
| `icon()` | an icon name like `"firefox"`, a file path, or empty |
| `image()` | a picture for this notification, like an avatar: a file path, a `file://` link, or empty |
| `urgency()` | `Urgency::Low`, `Normal`, or `Critical` |
| `actions()` | the buttons the sender asked for, in order |
| `has_default_action()` | whether clicking the notification itself does something |
| `received()` | when it arrived, or when the sender last replaced it, as a `SystemTime` |

Each `Action` has `key()`, to pass to `invoke`, and `label()`, the text to show on the button.

## Clicking, dismissing, and actions

| Function | Does |
|---|---|
| `Notifications::click(id)` | runs the default action, or dismisses it when there's none. Use it when the notification itself is clicked. |
| `Notifications::invoke(id, key)` | runs one of its actions. Use it for action buttons. |
| `Notifications::dismiss(id)` | closes it |
| `Notifications::clear()` | closes all of them |

After an action runs, the notification closes, unless the sender asked for it to stay.

## Action buttons

```rust,ignore
for action in notification.actions() {
    let id = notification.id();
    let key = String::from(action.key());

    buttons.push(Box::new(
        Rectangle::new()
            .width(100.0)
            .height(30.0)
            .on_click(move |_| Notifications::invoke(id, &key))
            .child(Text::new(action.label())),
    ));
}
```

Try it with:

```sh
notify-send -A yes=Yes -A no=No "question" "pick one"
```

## Timeouts

Notifications stay until they're dismissed. Amane ignores the timeout the sender asks for.

To hide popups after a few seconds, compare `received()` with the current time in a Service that polls every second, and only show the recent ones in your popup window. Keep the full list for a notification center.
