#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Link {
    #[default]
    Offline,

    Wired,

    Wifi,

    // vpn, bluetooth tethering and the rest
    Other,
}

impl Link {
    // networkmanager names the kind after the settings it uses
    pub fn from_type(kind: &str) -> Self {
        match kind {
            "802-3-ethernet" => Self::Wired,
            "802-11-wireless" => Self::Wifi,
            _ => Self::Other,
        }
    }
}
