use image::Rgba;
use imageproc::drawing::draw_line_segment_mut;
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
        let mut screenshot = primary.capture_image().unwrap();
        let cx = width as f32 / 2.0;
        let cy = height as f32 / 2.0;
        let red = Rgba([255u8, 0, 0, 1]);
        draw_line_segment_mut(&mut screenshot, (0.0, cy), (width as f32, cy), red);
        draw_line_segment_mut(&mut screenshot, (cx, 0.0), (cx, height as f32), red);

        screenshot.save("screenshot.png").unwrap();
    }
    println!("{} x {} size", width, height);
}
