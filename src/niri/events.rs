use std::io::{BufRead, BufReader, Lines};
use std::os::unix::net::UnixStream;

use serde_json::Value;

use super::{Event, socket, workspace};

pub struct Events {
    lines: Lines<BufReader<UnixStream>>,
}

// streams niri's events until niri exits, so call it from a service thread; none without niri
pub fn events() -> Option<Events> {
    let stream = socket::send("\"EventStream\"")?;

    let events = Events {
        lines: BufReader::new(stream).lines(),
    };

    Some(events)
}

impl Iterator for Events {
    type Item = Event;

    fn next(&mut self) -> Option<Event> {
        loop {
            let line = self.lines.next()?.ok()?;

            if let Some(event) = parse(&line) {
                return Some(event);
            }
        }
    }
}

fn parse(line: &str) -> Option<Event> {
    let value: Value = serde_json::from_str(line).ok()?;

    // every event is an object with one key, the event's name
    let (name, body) = value.as_object()?.iter().next()?;

    match name.as_str() {
        "WorkspacesChanged" => workspaces_changed(body),
        "WorkspaceActivated" => workspace_activated(body),
        "WorkspaceUrgencyChanged" => urgency_changed(body),
        "WindowsChanged" => windows_changed(body),
        "WindowOpenedOrChanged" => window_changed(body),
        "WindowClosed" => window_closed(body),
        _ => None,
    }
}

fn windows_changed(body: &Value) -> Option<Event> {
    let mut windows = Vec::new();

    for window in body["windows"].as_array()? {
        windows.push((window["id"].as_u64()?, window["workspace_id"].as_u64()));
    }

    Some(Event::Windows(windows))
}

fn window_changed(body: &Value) -> Option<Event> {
    let window = &body["window"];

    let id = window["id"].as_u64()?;

    // null while the window is on no workspace
    let workspace = window["workspace_id"].as_u64();

    Some(Event::WindowChanged { id, workspace })
}

fn window_closed(body: &Value) -> Option<Event> {
    let id = body["id"].as_u64()?;

    Some(Event::WindowClosed { id })
}

fn workspaces_changed(body: &Value) -> Option<Event> {
    let mut list = Vec::new();

    for value in body["workspaces"].as_array()? {
        list.push(workspace::parse(value)?);
    }

    Some(Event::Workspaces(list))
}

fn workspace_activated(body: &Value) -> Option<Event> {
    let id = body["id"].as_u64()?;

    let focused = body["focused"].as_bool()?;

    Some(Event::Activated { id, focused })
}

fn urgency_changed(body: &Value) -> Option<Event> {
    let id = body["id"].as_u64()?;

    let urgent = body["urgent"].as_bool()?;

    Some(Event::Urgent { id, urgent })
}
