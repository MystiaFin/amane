use std::collections::HashMap;
use std::env;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;

use serde_json::{Value, json};

use crate::Workspace;

// the niri events amane uses, everything else niri sends is skipped
pub enum Event {
    // the full list, sent first and again whenever one is added or removed
    Workspaces(Vec<Workspace>),

    // focused is false when it only became the one shown on its monitor
    Activated { id: i64, focused: bool },

    Urgent { id: i64, urgent: bool },

    // every window and the workspace it is on, if any; sent first
    Windows(Vec<(u64, Option<i64>)>),

    // a window opened, or moved to another workspace
    WindowChanged { id: u64, workspace: Option<i64> },

    WindowClosed { id: u64 },
}

// niri's events only say what changed, so the rest is remembered here
#[derive(Default)]
struct State {
    list: Vec<Workspace>,

    // every window by its id, and the workspace it is on
    windows: HashMap<u64, Option<i64>>,
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

pub fn focus_workspace(id: i64) {
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

// follows niri's events until niri exits, handing over the full list after each one
pub fn listen(mut on_change: impl FnMut(Vec<Workspace>)) {
    let Some(stream) = send("\"EventStream\"") else {
        return;
    };

    let mut state = State::default();

    for line in BufReader::new(stream).lines() {
        let Ok(line) = line else {
            return;
        };

        let Some(event) = parse(&line) else {
            continue;
        };

        state.apply(event);

        on_change(state.list.clone());
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
        windows.push((window["id"].as_u64()?, window["workspace_id"].as_i64()));
    }

    Some(Event::Windows(windows))
}

fn window_changed(body: &Value) -> Option<Event> {
    let window = &body["window"];

    let id = window["id"].as_u64()?;

    // null while the window is on no workspace
    let workspace = window["workspace_id"].as_i64();

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
    let id = body["id"].as_i64()?;

    let focused = body["focused"].as_bool()?;

    Some(Event::Activated { id, focused })
}

fn urgency_changed(body: &Value) -> Option<Event> {
    let id = body["id"].as_i64()?;

    let urgent = body["urgent"].as_bool()?;

    Some(Event::Urgent { id, urgent })
}

fn parse_workspace(value: &Value) -> Option<Workspace> {
    let index = value["idx"].as_u64()?;

    let workspace = Workspace {
        id: value["id"].as_i64()?,
        index: index as u32,

        // null for workspaces the user never named
        name: value["name"].as_str().map(String::from),
        output: value["output"].as_str().map(String::from),

        active: value["is_active"].as_bool()?,
        focused: value["is_focused"].as_bool()?,
        urgent: value["is_urgent"].as_bool()?,

        // counted from the window events, after every event
        windows: 0,
    };

    Some(workspace)
}

impl State {
    fn apply(&mut self, event: Event) {
        match event {
            Event::Workspaces(list) => self.list = list,
            Event::Activated { id, focused } => self.activate(id, focused),
            Event::Urgent { id, urgent } => self.mark_urgent(id, urgent),

            Event::Windows(windows) => {
                self.windows = windows.into_iter().collect();
            }

            Event::WindowChanged { id, workspace } => {
                self.windows.insert(id, workspace);
            }

            Event::WindowClosed { id } => {
                self.windows.remove(&id);
            }
        }

        self.count_windows();
    }

    fn activate(&mut self, id: i64, focused: bool) {
        let Some(activated) = self.list.iter().find(|workspace| workspace.id == id) else {
            return;
        };

        let output = activated.output.clone();

        // only one workspace is shown per monitor, and only one has focus overall
        for workspace in &mut self.list {
            if workspace.output == output {
                workspace.active = workspace.id == id;
            }

            if focused {
                workspace.focused = workspace.id == id;
            }
        }
    }

    fn mark_urgent(&mut self, id: i64, urgent: bool) {
        for workspace in &mut self.list {
            if workspace.id == id {
                workspace.urgent = urgent;
            }
        }
    }

    // a new workspace list starts at 0 windows each, so the counts are made again after any event
    fn count_windows(&mut self) {
        for workspace in &mut self.list {
            let mut count = 0;

            for on in self.windows.values() {
                if *on == Some(workspace.id) {
                    count += 1;
                }
            }

            workspace.windows = count;
        }
    }
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
