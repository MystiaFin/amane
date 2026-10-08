use amane::{
    Action, App, Center, Color, Column, Horizontal, Layer, LayerWindow, Notification,
    Notifications, Parent, Rectangle, Row, Service, Text, Urgency, Vertical, Widget,
};

/*
 * stop your notification daemon first (mako, dunst, a quickshell bar),
 * then send one with: notify-send "hello" "from amane"
 * or with buttons: notify-send -A yes=Yes -A no=No "question" "pick one"
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
        .child(Column::new(cards).gap(8.0))
}

// click a notification to run its default action, or to dismiss it
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

    let body = card(&label, fill, Some(notification.id()));

    if notification.actions().is_empty() {
        return body;
    }

    let mut buttons: Vec<Box<dyn Widget>> = Vec::new();

    for action in notification.actions() {
        buttons.push(action_button(notification.id(), action));
    }

    Box::new(Column::new(vec![
        body,
        Box::new(Row::new(buttons).gap(4.0)),
    ]))
}

fn action_button(id: u32, action: &Action) -> Box<dyn Widget> {
    let key = String::from(action.key());

    Box::new(
        Rectangle::new()
            .width(100.0)
            .height(30.0)
            .fill(Color::GREEN)
            .align_child(Center, Center)
            .on_click(move |_| Notifications::invoke(id, &key))
            .child(Text::new(action.label()).size(16.0).color(Color::WHITE)),
    )
}

fn card(label: &str, fill: Color, id: Option<u32>) -> Box<dyn Widget> {
    let mut rectangle = Rectangle::new()
        .width(Parent)
        .height(60.0)
        .fill(fill)
        .child(Text::new(label).size(16.0).color(Color::WHITE));

    if let Some(id) = id {
        rectangle = rectangle.on_click(move |_| Notifications::click(id));
    }

    Box::new(rectangle)
}
