use xcap::Monitor;

fn main() {
    let mut width = 0;
    let mut height = 0;
    let displays = Monitor::all().unwrap();
    if let Some(primary) = displays.first() {
        if let Ok(w) = primary.width()
            && let Ok(h) = primary.height()
        {
            width = w;
            height = h;
        }
        let screenshot = primary.capture_image().unwrap();
        screenshot.save("screenshot.png").unwrap();
    }
    println!("{} x {} size", width, height);
}
