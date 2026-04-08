use xcap::Monitor;

pub struct MonitorInfo {
    pub id: u32,
    pub width: u32,
    pub height: u32,
    pub is_primary: bool,
}

impl MonitorInfo {
    pub fn all() -> Vec<Self> {
        Monitor::all()
            .unwrap_or_default()
            .into_iter()
            .map(|m| Self {
                id: m.id().unwrap_or(0),
                width: m.width().unwrap_or(0),
                height: m.height().unwrap_or(0),
                is_primary: m.is_primary().unwrap_or(false),
            })
            .collect()
    }
}
