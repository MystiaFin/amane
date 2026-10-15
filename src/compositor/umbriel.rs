use std::env;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

use serde_json::{json, Value};

use crate::Workspace;

fn socket() -> Option<PathBuf> {
    if let Some(path) = env::var_os("UMBRIEL_SOCKET") {
        return Some(PathBuf::from(path));
    }

    let runtime = env::var_os("XDG_RUNTIME_DIR")?;
    let display = env::var_os("WAYLAND_DISPLAY")
        .map(|d| d.to_string_lossy().into_owned())
        .unwrap_or_else(|| "wayland-0".to_string());

    Some(PathBuf::from(runtime).join(format!("umbriel-{display}.sock")))
}

fn request(command: &str) -> Option<String> {
    let path = socket()?;
    let mut stream = UnixStream::connect(path).ok()?;

    let line = format!("{command}\n");
    stream.write_all(line.as_bytes()).ok()?;

    let mut reply = String::new();
    BufReader::new(stream).read_line(&mut reply).ok()?;

    Some(reply)
}

pub fn focus_workspace(id: i64) {
    let Some(target) = target_of(id) else {
        return;
    };

    let req = json!({
        "cmd": "msg",
        "arg": format!("workspace-switch:{target}")
    });

    let _ = request(&req.to_string());
}

fn target_of(id: i64) -> Option<String> {
    if id >= 0 {
        return Some(id.to_string());
    }

    name_of(id)
}

fn name_of(id: i64) -> Option<String> {
    let req = json!({ "cmd": "workspaces" });
    let reply = request(&req.to_string())?;
    let value: Value = serde_json::from_str(&reply).ok()?;

    let workspaces = value.get("ok").and_then(Value::as_array)
        .or_else(|| value.as_array())?;

    for workspace in workspaces {
        if workspace["id"].as_i64() == Some(id) {
            return workspace["name"].as_str().map(String::from);
        }
    }

    None
}

pub fn listen(mut on_change: impl FnMut(Vec<Workspace>)) {
    let Some(path) = socket() else {
        return;
    };

    let Ok(mut stream) = UnixStream::connect(path) else {
        return;
    };

    let req = json!({
        "cmd": "subscribe",
        "events": ["workspaces"]
    });

    let line = format!("{req}\n");
    if stream.write_all(line.as_bytes()).is_err() {
        return;
    }

    if let Some(list) = fetch() {
        on_change(list);
    }

    let reader = BufReader::new(stream);
    for line in reader.lines() {
        let Ok(line) = line else {
            return;
        };

        if let Some(list) = parse(&line) {
            on_change(list);
        }
    }
}

fn fetch() -> Option<Vec<Workspace>> {
    let req = json!({ "cmd": "workspaces" });
    let reply = request(&req.to_string())?;
    parse(&reply)
}

fn parse(line: &str) -> Option<Vec<Workspace>> {
    let value: Value = serde_json::from_str(line).ok()?;

    if let Some(event) = value.get("event").and_then(Value::as_str) {
        if event == "workspaces" {
            let data = value.get("data")?.as_array()?;
            return parse_workspaces(data);
        }
        return None;
    }

    if let Some(ok) = value.get("ok").and_then(Value::as_array) {
        return parse_workspaces(ok);
    }

    if let Some(array) = value.as_array() {
        return parse_workspaces(array);
    }

    None
}

fn parse_workspaces(workspaces: &[Value]) -> Option<Vec<Workspace>> {
    let mut list = Vec::new();

    for workspace in workspaces {
        let id = workspace["id"].as_i64()?;
        let index = workspace["index"]
            .as_u64()
            .or_else(|| workspace["idx"].as_u64())? as u32;

        let display_name = workspace["name"]
            .as_str()
            .or_else(|| workspace["display_name"].as_str());

        let named = workspace["named"]
            .as_bool()
            .unwrap_or_else(|| display_name.is_some_and(|n| n != index.to_string()));

        let name = if named {
            display_name.map(String::from)
        } else {
            None
        };

        let output = workspace["output"].as_str().map(String::from);
        let active = workspace["active"].as_bool().or_else(|| workspace["is_active"].as_bool())?;
        let focused = workspace["focused"].as_bool().or_else(|| workspace["is_focused"].as_bool())?;
        let urgent = workspace["urgent"].as_bool().or_else(|| workspace["is_urgent"].as_bool()).unwrap_or(false);
        let windows = workspace["window_count"].as_u64().or_else(|| workspace["windows"].as_u64()).unwrap_or(0) as u32;

        list.push(Workspace {
            id,
            index,
            name,
            output,
            active,
            focused,
            urgent,
            windows,
        });
    }

    Some(list)
}

#[cfg(test)]
mod tests {
    use super::*;

    const WORKSPACES_QUERY: &str = r#"{"ok":[{"id":1,"index":1,"name":"1","named":false,"output":"eDP-1","active":true,"focused":true,"urgent":false,"window_count":3},{"id":2,"index":2,"name":"web","named":true,"output":"eDP-1","active":false,"focused":false,"urgent":true,"window_count":1}]}"#;
    const WORKSPACE_EVENT: &str = r#"{"event":"workspaces","data":[{"id":1,"index":1,"name":"1","named":false,"output":"eDP-1","active":true,"focused":true,"urgent":false,"window_count":2}]}"#;

    #[test]
    fn reads_workspaces_from_query() {
        let list = parse(WORKSPACES_QUERY).expect("failed to parse query");
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].windows, 3);
        assert_eq!(list[1].name.as_deref(), Some("web"));
    }

        #[test]
    fn follows_event_stream_snapshots_until_eof() {
        let updated = r#"{"event":"workspaces","data":[{"id":1,"index":1,"name":"1","named":false,"output":"eDP-1","active":true,"focused":true,"urgent":false,"window_count":5}]}"#;
        let input = format!(
            "{WORKSPACE_EVENT}\n{{\"event\":\"windows\",\"data\":[]}}\ninvalid\n{updated}\n"
        );
        
        let mut updates = Vec::new();
        let reader = std::io::Cursor::new(input);
        
        for line in reader.lines().flatten() {
            if let Some(list) = parse(&line) {
                updates.push(list);
            }
        }

        assert_eq!(updates.len(), 2);
        assert_eq!(updates[0][0].windows, 2);
        assert_eq!(updates[1][0].windows, 5);
    }

}
