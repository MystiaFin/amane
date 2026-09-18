mod wayland;

use wayland::WaylandState;

fn main() {
    let connection =
        wayland::connection::connect();

    println!("Connected!");

    let mut event_queue =
        connection.new_event_queue();

    let qh =
        event_queue.handle();

    let mut state =
        WaylandState::new();

    let _registry =
        wayland::registry::request(
            &connection,
            &qh,
        );

    event_queue
        .roundtrip(&mut state)
        .expect("failed to process Wayland events");

    if state.surface.is_some() {
        println!("We have a surface!");
    }
}
