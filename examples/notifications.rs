use amane::{
    App, Color, Column, Horizontal, Layer, LayerWindow, Notification, Notifications, Parent,
    Rectangle, Service, Text, Urgency, Vertical, Widget,
};

/*
 * stop your notification daemon first (mako, dunst, a quickshell bar),
 * then send one with: notify-send "hello" "from amane"
 */
fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    let notifications = Notifications::read();

    let mut cards = Vec::new();

    if !notifications.running() {
        cards.push(card(
            "another notification daemon is running",
            Color::RED,
            None,
        ));
    }

    for notification in notifications.list() {
        cards.push(notification_card(notification));
    }

    LayerWindow::new()
        .width(400.0)
        .height(600.0)
        .anchor_vertical(Vertical::Top)
        .anchor_horizontal(Horizontal::Right)
        .layer(Layer::Top)
        .child(Column::new(cards))
}

// click a notification to dismiss it
fn notification_card(notification: &Notification) -> Box<dyn Widget> {
    let label = format!(
        "{}: {}  {}",
        notification.app_name(),
        notification.summary(),
        notification.body(),
    );

    let fill = match notification.urgency() {
        Urgency::Critical => Color::RED,
        _ => Color::BLUE,
    };

    card(&label, fill, Some(notification.id()))
}

fn card(label: &str, fill: Color, id: Option<u32>) -> Box<dyn Widget> {
    let mut rectangle = Rectangle::new()
        .width(Parent)
        .height(60.0)
        .fill(fill)
        .child(Text::new(label).size(16.0).color(Color::WHITE));

    if let Some(id) = id {
        rectangle = rectangle.on_click(move |_| Notifications::dismiss(id));
    }

    Box::new(rectangle)
}
