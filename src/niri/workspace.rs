use serde_json::Value;

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

pub fn parse(value: &Value) -> Option<NiriWorkspace> {
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
