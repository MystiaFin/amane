use std::env;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::sync::Mutex;

use serde_json::Value;

use crate::Workspace;

// monitor names keep their slots after unplugging, so an old id never points at another monitor
static OUTPUTS: Mutex<Vec<String>> = Mutex::new(Vec::new());

fn send(command: &str) -> Option<UnixStream> {
    let path = env::var_os("MANGO_INSTANCE_SIGNATURE")?;
    let mut stream = UnixStream::connect(path).ok()?;

    stream.write_all(format!("{command}\n").as_bytes()).ok()?;

    Some(stream)
}

fn request(command: &str) -> Option<String> {
    let stream = send(command)?;
    let mut reply = String::new();

    BufReader::new(stream).read_line(&mut reply).ok()?;

    Some(reply)
}

pub fn focus_workspace(id: i64) {
    let _ = focus(id, request);
}

fn focus(id: i64, mut request: impl FnMut(&str) -> Option<String>) -> Option<()> {
    let list = snapshot(&request("get all-monitors")?)?;
    let workspace = list.iter().find(|workspace| workspace.id == id)?;
    let output = workspace.output.as_deref()?;
    let selector = monitor_selector(output)?;
    let command = focus_command(id, &list)?;

    request(&format!("dispatch focusmon,{selector}"))?;

    // disabled outputs remain in snapshots; viewcrossmon would switch the current output instead
    let confirmed: Value = serde_json::from_str(&request("get all-monitors")?).ok()?;
    let selected = confirmed["monitors"].as_array()?.iter().any(|monitor| {
        monitor["name"].as_str() == Some(output) && monitor["active"].as_bool() == Some(true)
    });

    if !selected {
        return None;
    }

    request(&command)?;

    Some(())
}

fn focus_command(id: i64, list: &[Workspace]) -> Option<String> {
    let workspace = list.iter().find(|workspace| workspace.id == id)?;
    let output = workspace.output.as_deref()?;

    let selector = monitor_selector(output)?;

    Some(format!(
        "dispatch viewcrossmon,{},{}",
        workspace.index, selector
    ))
}

fn monitor_selector(output: &str) -> Option<String> {
    if output.contains([',', ':', '\n', '\r']) {
        return None;
    }

    // monitor selectors are PCRE2 patterns, so DP-1 must not match DP-10
    let mut selector = String::from("^");

    for character in output.chars() {
        if r"\.^$|?*+()[]{}".contains(character) {
            selector.push('\\');
        }

        selector.push(character);
    }

    selector.push('$');

    Some(selector)
}

pub fn listen(on_change: impl FnMut(Vec<Workspace>)) {
    let Some(stream) = send("watch all-monitors") else {
        return;
    };

    // the watch reply is the initial snapshot, followed by a full snapshot on each change
    events(BufReader::new(stream), on_change);
}

fn events(reader: impl BufRead, mut on_change: impl FnMut(Vec<Workspace>)) {
    for line in reader.lines() {
        let Ok(line) = line else {
            return;
        };

        if let Some(list) = snapshot(&line) {
            on_change(list);
        }
    }
}

fn snapshot(line: &str) -> Option<Vec<Workspace>> {
    let mut outputs = OUTPUTS.lock().ok()?;

    parse(line, &mut outputs)
}

fn parse(line: &str, outputs: &mut Vec<String>) -> Option<Vec<Workspace>> {
    let value: Value = serde_json::from_str(line).ok()?;
    let mut list = Vec::new();

    for monitor in value["monitors"].as_array()? {
        let output = monitor["name"].as_str()?;
        let focused = monitor["active"].as_bool()?;

        let slot = match outputs.iter().position(|name| name == output) {
            Some(slot) => slot,
            None => {
                outputs.push(output.to_string());
                outputs.len() - 1
            }
        };

        for tag in monitor["tags"].as_array()? {
            let index = u32::try_from(tag["index"].as_u64()?).ok()?;

            // Mango uses a 31-bit tag mask; zero means a special workspace, not a regular tag
            if !(1..=31).contains(&index) {
                return None;
            }

            let active = tag["is_active"].as_bool()?;

            list.push(Workspace {
                id: ((slot as i64) << 32) | i64::from(index),
                index,
                name: None,
                output: Some(output.to_string()),
                active,
                focused: focused && active,
                urgent: tag["is_urgent"].as_bool()?,
                windows: u32::try_from(tag["client_count"].as_u64()?).ok()?,
            });
        }
    }

    Some(list)
}

#[cfg(test)]
mod tests {
    use super::*;

    // trimmed from mmsg get all-monitors; several tags can be active on one monitor
    const MONITORS: &str = r#"{"monitors":[
        {"name":"DP-9","active":false,"tags":[
            {"index":1,"is_active":true,"is_urgent":false,"layout":"T","client_count":2},
            {"index":2,"is_active":false,"is_urgent":true,"layout":"T","client_count":1}
        ]},
        {"name":"eDP-1","active":true,"tags":[
            {"index":1,"is_active":true,"is_urgent":false,"layout":"T","client_count":1},
            {"index":2,"is_active":true,"is_urgent":false,"layout":"T","client_count":0},
            {"index":3,"is_active":false,"is_urgent":false,"layout":"T","client_count":0}
        ]}
    ]}"#;

    #[test]
    fn reads_tags_on_each_monitor() {
        let list = parse(MONITORS, &mut Vec::new()).expect("failed to parse mango monitors");

        assert_eq!(list.len(), 5);
        assert_eq!(list[0].index, 1);
        assert_eq!(list[0].output.as_deref(), Some("DP-9"));
        assert_eq!(list[0].name, None);
        assert_eq!(list[0].windows, 2);
        assert!(list[0].active);
        assert!(!list[0].focused);
        assert!(list[1].urgent);
        assert!(!list[1].active);
        assert_eq!(list[1].windows, 1);
        assert_ne!(list[0].id, list[2].id);

        assert!(list[2].active);
        assert!(list[2].focused);
        assert!(list[3].active);
        assert!(list[3].focused);
        assert_eq!(list[3].windows, 0);
        assert!(!list[4].active);
        assert!(!list[4].focused);
    }

    #[test]
    fn keeps_ids_when_monitors_reorder_or_disconnect() {
        let mut outputs = Vec::new();
        let original = parse(MONITORS, &mut outputs).unwrap();
        let mut value: serde_json::Value = serde_json::from_str(MONITORS).unwrap();
        let monitors = value["monitors"].as_array_mut().unwrap();
        monitors.reverse();

        let reordered = parse(&value.to_string(), &mut outputs).unwrap();
        assert_eq!(original[0].id, reordered[3].id);
        assert_eq!(original[2].id, reordered[0].id);

        value["monitors"].as_array_mut().unwrap().pop();
        let remaining = parse(&value.to_string(), &mut outputs).unwrap();
        assert_eq!(original[2].id, remaining[0].id);
        assert!(focus_command(original[0].id, &remaining).is_none());

        let reconnected = parse(MONITORS, &mut outputs).unwrap();
        assert_eq!(original[0].id, reconnected[0].id);
    }

    #[test]
    fn focuses_the_tag_on_its_own_monitor() {
        let list = parse(MONITORS, &mut Vec::new()).unwrap();

        assert_eq!(
            focus_command(list[1].id, &list).as_deref(),
            Some("dispatch viewcrossmon,2,^DP-9$")
        );
        assert_eq!(
            focus_command(list[3].id, &list).as_deref(),
            Some("dispatch viewcrossmon,2,^eDP-1$")
        );
        assert!(focus_command(-1, &list).is_none());
        assert!(focus_command(i64::MAX, &list).is_none());
    }

    #[test]
    fn accepts_no_monitors_and_skips_invalid_replies() {
        let mut outputs = Vec::new();

        assert!(
            parse(r#"{"monitors":[]}"#, &mut outputs)
                .unwrap()
                .is_empty()
        );

        for line in [
            "not json",
            r#"{"error":"unknown command"}"#,
            r#"{"monitors":[{"name":"DP-9","active":true}]}"#,
        ] {
            assert!(parse(line, &mut outputs).is_none());
        }
    }

    #[test]
    fn follows_initial_state_and_later_events_until_eof() {
        let first: serde_json::Value = serde_json::from_str(MONITORS).unwrap();
        let changed = r#"{"monitors":[{"name":"eDP-1","active":true,"tags":[
            {"index":2,"is_active":true,"is_urgent":false,"client_count":3}
        ]}]}"#;
        let input = format!("{first}\ninvalid\n{}\n", changed.replace('\n', ""));
        let mut updates = Vec::new();

        events(std::io::Cursor::new(input), |list| updates.push(list));

        assert_eq!(updates.len(), 2);
        assert_eq!(updates[0].len(), 5);
        assert_eq!(updates[1].len(), 1);
        assert_eq!(updates[1][0].index, 2);
        assert_eq!(updates[1][0].windows, 3);
        assert!(updates[1][0].focused);
    }

    #[test]
    fn rejects_invalid_tag_numbers_and_window_counts() {
        for (field, number) in [
            ("index", 0),
            ("index", 32),
            ("client_count", -1),
            ("client_count", 4_294_967_296_i64),
        ] {
            let mut value: serde_json::Value = serde_json::from_str(MONITORS).unwrap();
            value["monitors"][0]["tags"][0][field] = number.into();

            assert!(parse(&value.to_string(), &mut Vec::new()).is_none());
        }
    }

    #[test]
    fn rejects_monitor_names_that_change_command_arguments() {
        let mut list = parse(MONITORS, &mut Vec::new()).unwrap();

        for name in ["DP-9,1", "DP-9:1", "DP-9\ndispatch view,2", "DP-9\r"] {
            list[0].output = Some(name.to_string());
            assert!(focus_command(list[0].id, &list).is_none());
        }
    }

    #[test]
    fn matches_monitor_names_exactly_instead_of_as_regexes() {
        let mut list = parse(MONITORS, &mut Vec::new()).unwrap();

        for (name, command) in [
            ("DP-1", "dispatch viewcrossmon,1,^DP-1$"),
            ("DP-10", "dispatch viewcrossmon,1,^DP-10$"),
            (
                "Virtual.1+(left)",
                r"dispatch viewcrossmon,1,^Virtual\.1\+\(left\)$",
            ),
            (r"Virtual\[1]", r"dispatch viewcrossmon,1,^Virtual\\\[1\]$"),
        ] {
            list[0].output = Some(name.to_string());
            assert_eq!(focus_command(list[0].id, &list).as_deref(), Some(command));
        }
    }

    #[test]
    fn verifies_monitor_selection_before_switching_tags() {
        let list = snapshot(MONITORS).unwrap();
        let mut selected: Value = serde_json::from_str(MONITORS).unwrap();
        selected["monitors"][0]["active"] = true.into();
        selected["monitors"][1]["active"] = false.into();
        let mut commands = Vec::new();

        let result = focus(list[1].id, |command| {
            commands.push(command.to_string());
            match commands.len() {
                1 => Some(MONITORS.to_string()),
                3 => Some(selected.to_string()),
                _ => Some(r#"{"success":true}"#.to_string()),
            }
        });

        assert_eq!(result, Some(()));
        assert_eq!(
            commands,
            [
                "get all-monitors",
                "dispatch focusmon,^DP-9$",
                "get all-monitors",
                "dispatch viewcrossmon,2,^DP-9$",
            ]
        );
    }

    #[test]
    fn does_not_switch_tags_when_target_monitor_is_disabled() {
        let list = snapshot(MONITORS).unwrap();
        let mut commands = Vec::new();

        let result = focus(list[1].id, |command| {
            commands.push(command.to_string());
            if command == "get all-monitors" {
                Some(MONITORS.to_string())
            } else {
                Some(r#"{"success":true}"#.to_string())
            }
        });

        assert!(result.is_none());
        assert_eq!(
            commands,
            [
                "get all-monitors",
                "dispatch focusmon,^DP-9$",
                "get all-monitors",
            ]
        );
    }
}
