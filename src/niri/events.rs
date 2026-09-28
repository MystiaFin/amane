use std::io::{BufRead, BufReader, Lines};
use std::os::unix::net::UnixStream;

use serde_json::Value;

use super::{Event, socket, workspace};

pub struct Events {
    lines: Lines<BufReader<UnixStream>>,
}

// streams niri's events until niri exits, so call it from a service thread
pub fn events() -> Events {
    let stream = socket::send("\"EventStream\"");

    Events {
        lines: BufReader::new(stream).lines(),
    }
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
        _ => None,
    }
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
