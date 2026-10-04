use std::collections::HashMap;
use std::env;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;

use serde_json::Value;

use crate::Workspace;

// message types from sway's ipc, the i3 protocol
const RUN_COMMAND: u32 = 0;
const GET_WORKSPACES: u32 = 1;
const SUBSCRIBE: u32 = 2;
const GET_TREE: u32 = 4;

fn connect() -> Option<UnixStream> {
    let path = env::var_os("SWAYSOCK")?;

    UnixStream::connect(path).ok()
}

// every message is "i3-ipc", the payload's length, the message type, then the payload
fn send(stream: &mut UnixStream, kind: u32, payload: &str) -> Option<()> {
    let length = payload.len() as u32;

    let mut message = Vec::new();

    message.extend_from_slice(b"i3-ipc");
    message.extend_from_slice(&length.to_ne_bytes());
    message.extend_from_slice(&kind.to_ne_bytes());
    message.extend_from_slice(payload.as_bytes());

    stream.write_all(&message).ok()
}

fn receive(stream: &mut UnixStream) -> Option<String> {
    let mut header = [0; 14];

    stream.read_exact(&mut header).ok()?;

    // the 4 bytes after "i3-ipc"
    let length = u32::from_ne_bytes(header[6..10].try_into().ok()?);

    let mut payload = vec![0; length as usize];

    stream.read_exact(&mut payload).ok()?;

    String::from_utf8(payload).ok()
}

fn request(kind: u32, payload: &str) -> Option<String> {
    let mut stream = connect()?;

    send(&mut stream, kind, payload)?;

    receive(&mut stream)
}

pub fn focus_workspace(id: u64) {
    let Some(name) = name_of(id) else {
        return;
    };

    // quotes keep a name with spaces in one piece
    let command = format!("workspace \"{}\"", name.replace('"', "\\\""));

    let _ = request(RUN_COMMAND, &command);
}

// commands take a workspace's name, not its id
fn name_of(id: u64) -> Option<String> {
    let reply = request(GET_WORKSPACES, "")?;

    let workspaces: Value = serde_json::from_str(&reply).ok()?;

    for workspace in workspaces.as_array()? {
        if workspace["id"].as_u64() == Some(id) {
            return workspace["name"].as_str().map(String::from);
        }
    }

    None
}

/*
 * sway's events describe one change, and a window event doesn't say
 * which workspace it touched, so the full list is asked for again after each one
 */
pub fn listen(mut on_change: impl FnMut(Vec<Workspace>)) {
    let Some(mut stream) = connect() else {
        return;
    };

    if send(&mut stream, SUBSCRIBE, r#"["workspace","window"]"#).is_none() {
        return;
    }

    loop {
        if let Some(list) = fetch() {
            on_change(list);
        }

        // the subscribe reply comes first, then one message per event; stops when sway exits
        if receive(&mut stream).is_none() {
            return;
        }
    }
}

fn fetch() -> Option<Vec<Workspace>> {
    let workspaces = request(GET_WORKSPACES, "")?;
    let tree = request(GET_TREE, "")?;

    parse(&workspaces, &tree)
}

fn parse(workspaces: &str, tree: &str) -> Option<Vec<Workspace>> {
    let workspaces: Value = serde_json::from_str(workspaces).ok()?;
    let tree: Value = serde_json::from_str(tree).ok()?;

    // the workspace list has no window counts, the tree does: root, then outputs, then workspaces
    let mut counts = HashMap::new();

    for output in tree["nodes"].as_array()? {
        for workspace in output["nodes"].as_array()? {
            let id = workspace["id"].as_u64()?;

            counts.insert(id, count_windows(workspace));
        }
    }

    let mut list = Vec::new();

    for workspace in workspaces.as_array()? {
        let id = workspace["id"].as_u64()?;
        let number = workspace["num"].as_i64()?;
        let name = workspace["name"].as_str()?;

        // unnamed workspaces are named after their number
        let named = name != number.to_string();

        let windows = counts.get(&id).copied().unwrap_or(0);

        list.push(Workspace {
            id,

            // the number users see; a workspace with only a name has -1, kept as 0
            index: number.max(0) as u32,

            name: named.then(|| name.to_string()),
            output: workspace["output"].as_str().map(String::from),

            active: workspace["visible"].as_bool()?,
            focused: workspace["focused"].as_bool()?,
            urgent: workspace["urgent"].as_bool()?,

            windows,
        });
    }

    Some(list)
}

// a window is a container with nothing inside; split containers only hold other containers
fn count_windows(node: &Value) -> u32 {
    let mut count = 0;

    for child in children(node) {
        if children(child).is_empty() {
            count += 1;
        } else {
            count += count_windows(child);
        }
    }

    count
}

fn children(node: &Value) -> Vec<&Value> {
    let mut children = Vec::new();

    for key in ["nodes", "floating_nodes"] {
        if let Some(list) = node[key].as_array() {
            children.extend(list);
        }
    }

    children
}

#[cfg(test)]
mod tests {
    use super::*;

    // replies trimmed from swaymsg -t get_workspaces and swaymsg -t get_tree
    #[test]
    fn reads_workspaces_and_counts_windows() {
        let workspaces = r#"[
            {"id":4,"num":1,"name":"1","output":"eDP-1","visible":true,"focused":true,"urgent":false},
            {"id":9,"num":3,"name":"3:web","output":"eDP-1","visible":false,"focused":false,"urgent":true}
        ]"#;

        // workspace 1 has a split holding two windows, plus one floating window
        let tree = r#"{"nodes":[
            {"name":"__i3","nodes":[{"id":2,"name":"__i3_scratch","nodes":[],"floating_nodes":[]}]},
            {"name":"eDP-1","nodes":[
                {"id":4,"nodes":[
                    {"id":5,"nodes":[
                        {"id":6,"nodes":[],"floating_nodes":[]},
                        {"id":7,"nodes":[],"floating_nodes":[]}
                    ],"floating_nodes":[]}
                ],"floating_nodes":[
                    {"id":8,"nodes":[],"floating_nodes":[]}
                ]},
                {"id":9,"nodes":[],"floating_nodes":[]}
            ]}
        ]}"#;

        let list = parse(workspaces, tree).expect("failed to parse sway replies");

        assert_eq!(list.len(), 2);

        assert_eq!(list[0].name, None);
        assert_eq!(list[0].windows, 3);
        assert!(list[0].focused);

        assert_eq!(list[1].index, 3);
        assert_eq!(list[1].name.as_deref(), Some("3:web"));
        assert_eq!(list[1].windows, 0);
        assert!(list[1].urgent);
    }
}
