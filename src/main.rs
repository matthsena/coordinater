use screen_size::get_primary_screen_size;
use xcap::Monitor;

fn main() {
    let mut width = 0;
    let mut height = 0;
    if let Ok((w, h)) = get_primary_screen_size() {
        width = w;
        height = h;
    }
    print!("{} {}", width, height);
    // Screenshots
    let displays = Monitor::all().unwrap();
    if let Some(primary) = displays.first() {
        let screenshot = primary.capture_image().unwrap();
        screenshot.save("screenshot.png").unwrap();
    }
}
