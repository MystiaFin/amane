use std::env;
use std::io::{BufRead, BufReader, Lines, Write};
use std::os::unix::net::UnixStream;

use serde_json::{Value, json};

// a workspace as niri reports it
pub struct NiriWorkspace {
    pub id: u64,

    // position on its monitor, starting at 1
    pub index: u32,

    // null for workspaces the user never named
    pub name: Option<String>,
    pub output: Option<String>,

    pub active: bool,
    pub focused: bool,
    pub urgent: bool,
}

// the niri events amane uses, everything else niri sends is skipped
pub enum Event {
    // the full list, sent first and again whenever one is added or removed
    Workspaces(Vec<NiriWorkspace>),

    // focused is false when it only became the one shown on its monitor
    Activated { id: u64, focused: bool },

    Urgent { id: u64, urgent: bool },

    // every window and the workspace it is on, if any; sent first
    Windows(Vec<(u64, Option<u64>)>),

    // a window opened, or moved to another workspace
    WindowChanged { id: u64, workspace: Option<u64> },

    WindowClosed { id: u64 },
}

pub struct Events {
    lines: Lines<BufReader<UnixStream>>,
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

/*
 * niri reads one json request per line, then answers on the same connection;
 * none when niri is not running, like under another compositor
 */
pub fn send(request: &str) -> Option<UnixStream> {
    let path = env::var("NIRI_SOCKET").ok()?;

    let mut stream = UnixStream::connect(path).ok()?;

    let line = format!("{request}\n");

    stream.write_all(line.as_bytes()).ok()?;

    Some(stream)
}

pub fn focus_workspace(id: u64) {
    let request = json!({
        "Action": {
            "FocusWorkspace": {
                "reference": { "Id": id }
            }
        }
    });

    let Some(stream) = send(&request.to_string()) else {
        return;
    };

    // waiting for the one-line answer keeps niri from seeing a connection that closed mid-request
    let mut reply = String::new();

    let _ = BufReader::new(stream).read_line(&mut reply);
}

// streams niri's events until niri exits, so call it from a service thread; none without niri
pub fn events() -> Option<Events> {
    let stream = send("\"EventStream\"")?;

    let events = Events {
        lines: BufReader::new(stream).lines(),
    };

    Some(events)
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
        list.push(parse_workspace(value)?);
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

fn parse_workspace(value: &Value) -> Option<NiriWorkspace> {
    let index = value["idx"].as_u64()?;

    let workspace = NiriWorkspace {
        id: value["id"].as_u64()?,
        index: index as u32,

        name: value["name"].as_str().map(String::from),
        output: value["output"].as_str().map(String::from),

        active: value["is_active"].as_bool()?,
        focused: value["is_focused"].as_bool()?,
        urgent: value["is_urgent"].as_bool()?,
    };

    Some(workspace)
}

#[cfg(test)]
mod tests {
    use super::*;

    // lines copied from niri's event stream, trimmed to the fields amane reads plus a few it skips
    #[test]
    fn reads_niri_event_lines() {
        let line = r#"{"WorkspacesChanged":{"workspaces":[{"id":3,"idx":1,"name":null,"output":"eDP-1","is_urgent":false,"is_active":true,"is_focused":true,"active_window_id":null}]}}"#;

        let Some(Event::Workspaces(list)) = parse(line) else {
            panic!("failed to parse workspaces");
        };

        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, 3);
        assert_eq!(list[0].index, 1);
        assert_eq!(list[0].name, None);
        assert!(list[0].focused);

        let line = r#"{"WindowOpenedOrChanged":{"window":{"id":7,"title":"foot","app_id":"foot","workspace_id":null,"is_focused":false}}}"#;

        assert!(matches!(parse(line), Some(Event::WindowChanged { id: 7, workspace: None })));

        // events amane doesn't use are skipped
        assert!(parse(r#"{"KeyboardLayoutsChanged":{"keyboard_layouts":{"names":[],"current_idx":0}}}"#).is_none());
    }
}
