use serde_json::Value;

use crate::Workspace;

pub fn parse(value: &Value) -> Option<Workspace> {
    let index = value["idx"].as_u64()?;

    let workspace = Workspace {
        id: value["id"].as_u64()?,
        index: index as u32,

        // null for workspaces the user never named
        name: value["name"].as_str().map(String::from),
        output: value["output"].as_str().map(String::from),

        active: value["is_active"].as_bool()?,
        focused: value["is_focused"].as_bool()?,
        urgent: value["is_urgent"].as_bool()?,
    };

    Some(workspace)
}
