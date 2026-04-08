use xcap::Monitor;

pub struct MonitorInfo {
    pub id: u32,
    pub x: i32,
    pub y: i32,
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
                x: m.x().unwrap_or(0),
                y: m.y().unwrap_or(0),
                width: m.width().unwrap_or(0),
                height: m.height().unwrap_or(0),
                is_primary: m.is_primary().unwrap_or(false),
            })
            .collect()
    }

    pub fn find_by_id(monitors: &[MonitorInfo], id: u32) -> Option<&MonitorInfo> {
        monitors.iter().find(|m| m.id == id)
    }

    pub fn find_primary(monitors: &[MonitorInfo]) -> Option<&MonitorInfo> {
        monitors.iter().find(|m| m.is_primary)
    }

    pub fn resolve(monitors: &[MonitorInfo], id: Option<u32>) -> Result<&MonitorInfo, String> {
        match id {
            Some(id) => MonitorInfo::find_by_id(monitors, id).ok_or_else(|| {
                let available: Vec<String> = monitors.iter().map(|m| m.id.to_string()).collect();
                format!("monitor {} not found (available: {})", id, available.join(", "))
            }),
            None => MonitorInfo::find_primary(monitors).ok_or_else(|| {
                "no primary monitor found".to_string()
            }),
        }
    }

    pub fn validate_coords(&self, x: i32, y: i32) -> Result<(), String> {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            Err(format!(
                "coordinates ({},{}) out of bounds for monitor {} ({}x{})",
                x, y, self.id, self.width, self.height
            ))
        } else {
            Ok(())
        }
    }

    pub fn to_absolute(&self, x: i32, y: i32) -> (i32, i32) {
        (self.x + x, self.y + y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_monitor(id: u32, x: i32, y: i32, width: u32, height: u32, is_primary: bool) -> MonitorInfo {
        MonitorInfo { id, x, y, width, height, is_primary }
    }

    #[test]
    fn test_validate_coords_within_bounds() {
        let m = make_monitor(1, 0, 0, 1920, 1080, true);
        assert!(m.validate_coords(0, 0).is_ok());
        assert!(m.validate_coords(1919, 1079).is_ok());
        assert!(m.validate_coords(960, 540).is_ok());
    }

    #[test]
    fn test_validate_coords_out_of_bounds() {
        let m = make_monitor(1, 0, 0, 1920, 1080, true);
        assert!(m.validate_coords(1920, 0).is_err());
        assert!(m.validate_coords(0, 1080).is_err());
        assert!(m.validate_coords(-1, 0).is_err());
        assert!(m.validate_coords(0, -1).is_err());
        assert!(m.validate_coords(2000, 500).is_err());
    }

    #[test]
    fn test_to_absolute() {
        let m = make_monitor(1, 1920, 0, 2560, 1440, false);
        let (ax, ay) = m.to_absolute(100, 200);
        assert_eq!(ax, 2020);
        assert_eq!(ay, 200);
    }

    #[test]
    fn test_find_by_id() {
        let monitors = vec![
            make_monitor(1, 0, 0, 1920, 1080, true),
            make_monitor(2, 1920, 0, 2560, 1440, false),
        ];
        assert!(MonitorInfo::find_by_id(&monitors, 1).is_some());
        assert!(MonitorInfo::find_by_id(&monitors, 2).is_some());
        assert!(MonitorInfo::find_by_id(&monitors, 3).is_none());
    }

    #[test]
    fn test_find_primary() {
        let monitors = vec![
            make_monitor(1, 0, 0, 1920, 1080, true),
            make_monitor(2, 1920, 0, 2560, 1440, false),
        ];
        let primary = MonitorInfo::find_primary(&monitors).unwrap();
        assert_eq!(primary.id, 1);
    }
}
