use amane::{
    App, Center, Color, Column, LayerWindow, Lock, Monitor, Parent, Rectangle, Service, Text,
    TextInput, children,
};

const BACKGROUND: Color = Color::rgb(0x1e, 0x1e, 0x2e);

const FOREGROUND: Color = Color::rgb(0xcd, 0xd6, 0xf4);

/*
 * locks the session right away: click the box, type your password, press enter;
 * try it from a second tty first, a locked session only opens with the right password
 */
fn main() {
    Lock::start();

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

    // the compositor sizes lock screens to the monitor, so these are never used
    LayerWindow::new().width(1.0).height(1.0).child(
        Rectangle::new()
            .width(Parent)
            .height(Parent)
            .fill(BACKGROUND)
            .align_child(Center, Center)
            .child(
                Column::new(children![
                    Text::new(status).size(24.0).color(FOREGROUND),
                    Rectangle::new()
                        .width(300.0)
                        .height(40.0)
                        .fill(Color::WHITE)
                        .padding(8.0)
                        .child(
                            TextInput::new("password")
                                .size(20.0)
                                .placeholder("password")
                                .password()
                                .on_submit(|password| Lock::unlock(&password)),
                        ),
                ])
                .gap(12.0),
            ),
    )
}
