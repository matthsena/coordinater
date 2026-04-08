use xcap::Monitor;

pub fn capture(monitor_id: u32, output_path: &str) -> Result<(), String> {
    let monitors = Monitor::all()
        .map_err(|e| format!("failed to list monitors: {}", e))?;

    let monitor = monitors
        .into_iter()
        .find(|m| m.id().unwrap_or(0) == monitor_id)
        .ok_or_else(|| format!("monitor {} not found", monitor_id))?;

    let image = monitor
        .capture_image()
        .map_err(|e| format!("failed to capture screen: {}", e))?;

    image
        .save(output_path)
        .map_err(|e| format!("failed to save screenshot: {}", e))
}
