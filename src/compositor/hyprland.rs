use std::env;
use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

use serde_json::Value;

use crate::Workspace;

// hyprland keeps its sockets in a folder named after the running instance
fn socket(name: &str) -> Option<PathBuf> {
    let runtime = env::var_os("XDG_RUNTIME_DIR")?;
    let instance = env::var_os("HYPRLAND_INSTANCE_SIGNATURE")?;

    let path = PathBuf::from(runtime).join("hypr").join(instance).join(name);

    Some(path)
}

// one request per connection: hyprland answers, then closes it
fn request(command: &str) -> Option<String> {
    let path = socket(".socket.sock")?;

    let mut stream = UnixStream::connect(path).ok()?;

    stream.write_all(command.as_bytes()).ok()?;

    let mut reply = String::new();

    stream.read_to_string(&mut reply).ok()?;

    Some(reply)
}

pub fn focus_workspace(id: u64) {
    let _ = request(&format!("dispatch workspace {id}"));
}

/*
 * hyprland's event lines only name what happened, like workspace>>3,
 * so the full list is asked for again after each one
 */
pub fn listen(mut on_change: impl FnMut(Vec<Workspace>)) {
    let Some(path) = socket(".socket2.sock") else {
        return;
    };

    let Ok(stream) = UnixStream::connect(path) else {
        return;
    };

    let mut events = BufReader::new(stream).lines();

    loop {
        if let Some(list) = fetch() {
            on_change(list);
        }

        // waits for the next event, and stops when hyprland exits
        let Some(Ok(_)) = events.next() else {
            return;
        };
    }
}

fn fetch() -> Option<Vec<Workspace>> {
    let workspaces = request("j/workspaces")?;
    let monitors = request("j/monitors")?;

    parse(&workspaces, &monitors)
}

fn parse(workspaces: &str, monitors: &str) -> Option<Vec<Workspace>> {
    let workspaces: Value = serde_json::from_str(workspaces).ok()?;
    let monitors: Value = serde_json::from_str(monitors).ok()?;

    // every monitor shows one workspace, and the focused monitor's one has focus
    let mut shown = Vec::new();
    let mut focused = None;

    for monitor in monitors.as_array()? {
        let id = monitor["activeWorkspace"]["id"].as_i64()?;

        shown.push(id);

        if monitor["focused"].as_bool()? {
            focused = Some(id);
        }
    }

    let mut list = Vec::new();

    for workspace in workspaces.as_array()? {
        let id = workspace["id"].as_i64()?;

        // special workspaces, like the scratchpad, have negative ids and no place in a bar
        if id < 0 {
            continue;
        }

        // unnamed workspaces are named after their id
        let name = workspace["name"].as_str()?;

        let named = name != id.to_string();

        let windows = workspace["windows"].as_u64()?;

        list.push(Workspace {
            id: id as u64,

            // hyprland numbers workspaces across all monitors, and that number is what users see
            index: id as u32,

            name: named.then(|| name.to_string()),
            output: workspace["monitor"].as_str().map(String::from),

            active: shown.contains(&id),
            focused: focused == Some(id),

            // hyprland reports urgent windows, not workspaces
            urgent: false,

            windows: windows as u32,
        });
    }

    Some(list)
}

#[cfg(test)]
mod tests {
    use super::*;

    // replies trimmed from hyprctl -j workspaces and hyprctl -j monitors
    #[test]
    fn reads_workspaces_and_monitors() {
        let workspaces = r#"[
            {"id":1,"name":"1","monitor":"eDP-1","windows":2},
            {"id":2,"name":"web","monitor":"HDMI-A-1","windows":0},
            {"id":4,"name":"4","monitor":"eDP-1","windows":1},
            {"id":-98,"name":"special:scratchpad","monitor":"eDP-1","windows":1}
        ]"#;

        let monitors = r#"[
            {"name":"eDP-1","activeWorkspace":{"id":4,"name":"4"},"focused":true},
            {"name":"HDMI-A-1","activeWorkspace":{"id":2,"name":"web"},"focused":false}
        ]"#;

        let list = parse(workspaces, monitors).expect("failed to parse hyprland replies");

        assert_eq!(list.len(), 3);

        assert_eq!(list[0].name, None);
        assert_eq!(list[0].windows, 2);
        assert!(!list[0].active);

        assert_eq!(list[1].name.as_deref(), Some("web"));
        assert!(list[1].active);
        assert!(!list[1].focused);

        assert_eq!(list[2].index, 4);
        assert!(list[2].focused);
    }
}
