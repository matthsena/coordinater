use std::num::NonZeroU32;
use std::rc::Rc;
use std::thread;
use std::time::{Duration, Instant};

use softbuffer::Surface;
use tiny_skia::{Color, Paint, PathBuilder, Pixmap, Stroke, Transform};
use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId, WindowLevel};

#[derive(Debug, Clone)]
pub enum Shape {
    Line { x1: f32, y1: f32, x2: f32, y2: f32 },
    Rect { x: f32, y: f32, width: f32, height: f32 },
    Circle { x: f32, y: f32, radius: f32 },
}

pub fn parse_color(name: &str) -> Result<Color, String> {
    match name.to_lowercase().as_str() {
        "red" => Ok(Color::from_rgba8(255, 0, 0, 255)),
        "green" => Ok(Color::from_rgba8(0, 255, 0, 255)),
        "blue" => Ok(Color::from_rgba8(0, 0, 255, 255)),
        "yellow" => Ok(Color::from_rgba8(255, 255, 0, 255)),
        "white" => Ok(Color::from_rgba8(255, 255, 255, 255)),
        _ => Err(format!("Unknown color: {}", name)),
    }
}

fn render_shape(shape: &Shape, color: Color, width: u32, height: u32) -> Option<Pixmap> {
    let mut pixmap = Pixmap::new(width, height)?;

    let mut paint = Paint::default();
    paint.set_color(color);
    paint.anti_alias = true;

    let mut stroke = Stroke::default();
    stroke.width = 3.0;

    let path = match shape {
        Shape::Line { x1, y1, x2, y2 } => {
            let mut pb = PathBuilder::new();
            pb.move_to(*x1, *y1);
            pb.line_to(*x2, *y2);
            pb.finish()?
        }
        Shape::Rect { x, y, width, height } => {
            let mut pb = PathBuilder::new();
            pb.move_to(*x, *y);
            pb.line_to(x + width, *y);
            pb.line_to(x + width, y + height);
            pb.line_to(*x, y + height);
            pb.close();
            pb.finish()?
        }
        Shape::Circle { x, y, radius } => {
            let mut pb = PathBuilder::new();
            let cx = *x;
            let cy = *y;
            let r = *radius;
            let k: f32 = 0.5522848;
            let kr = k * r;

            // Approximate circle with 4 cubic bezier curves
            pb.move_to(cx, cy - r);
            pb.cubic_to(cx + kr, cy - r, cx + r, cy - kr, cx + r, cy);
            pb.cubic_to(cx + r, cy + kr, cx + kr, cy + r, cx, cy + r);
            pb.cubic_to(cx - kr, cy + r, cx - r, cy + kr, cx - r, cy);
            pb.cubic_to(cx - r, cy - kr, cx - kr, cy - r, cx, cy - r);
            pb.close();
            pb.finish()?
        }
    };

    pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
    Some(pixmap)
}

struct OverlayApp {
    shape: Shape,
    color: Color,
    duration: Duration,
    start_time: Option<Instant>,
    monitor_x: i32,
    monitor_y: i32,
    monitor_width: u32,
    monitor_height: u32,
    window: Option<Rc<Window>>,
    context: Option<softbuffer::Context<Rc<Window>>>,
    surface: Option<Surface<Rc<Window>, Rc<Window>>>,
}

impl ApplicationHandler for OverlayApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attrs = Window::default_attributes()
            .with_title("Overlay")
            .with_decorations(false)
            .with_transparent(true)
            .with_window_level(WindowLevel::AlwaysOnTop)
            .with_position(PhysicalPosition::new(self.monitor_x, self.monitor_y))
            .with_inner_size(PhysicalSize::new(self.monitor_width, self.monitor_height));

        let window = Rc::new(event_loop.create_window(attrs).unwrap());

        let context = softbuffer::Context::new(window.clone()).unwrap();
        let surface = Surface::new(&context, window.clone()).unwrap();

        // Make the overlay click-through so mouse events pass to windows below
        window.set_cursor_hittest(false).ok();

        self.window = Some(window);
        self.context = Some(context);
        self.surface = Some(surface);
        self.start_time = Some(Instant::now());
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::RedrawRequested => {
                // Check timeout
                if let Some(start) = self.start_time {
                    if start.elapsed() >= self.duration {
                        event_loop.exit();
                        return;
                    }
                }

                let window = self.window.as_ref().unwrap();
                let size = window.inner_size();
                let width = size.width;
                let height = size.height;

                if width == 0 || height == 0 {
                    return;
                }

                let surface = self.surface.as_mut().unwrap();
                surface
                    .resize(
                        NonZeroU32::new(width).unwrap(),
                        NonZeroU32::new(height).unwrap(),
                    )
                    .unwrap();

                let Some(pixmap) = render_shape(&self.shape, self.color, width, height) else {
                    return;
                };

                let mut buffer = surface.buffer_mut().unwrap();
                let pixels = pixmap.data();
                for i in 0..(width * height) as usize {
                    let r = pixels[i * 4] as u32;
                    let g = pixels[i * 4 + 1] as u32;
                    let b = pixels[i * 4 + 2] as u32;
                    let a = pixels[i * 4 + 3] as u32;
                    // softbuffer expects 0x00RRGGBB (or with alpha in high byte on some platforms)
                    // Pre-multiply with alpha for transparent background
                    buffer[i] = (a << 24) | (r << 16) | (g << 8) | b;
                }
                buffer.present().unwrap();

                // Throttle to ~10fps to avoid burning CPU on a static overlay
                thread::sleep(Duration::from_millis(100));
                window.request_redraw();
            }
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            _ => {}
        }
    }
}

pub fn show_overlay(
    shape: Shape,
    color: Color,
    duration_secs: f64,
    monitor_x: i32,
    monitor_y: i32,
    monitor_width: u32,
    monitor_height: u32,
) -> Result<(), String> {
    let event_loop = EventLoop::new().map_err(|e| format!("Failed to create event loop: {}", e))?;

    let mut app = OverlayApp {
        shape,
        color,
        duration: Duration::from_secs_f64(duration_secs),
        start_time: None,
        monitor_x,
        monitor_y,
        monitor_width,
        monitor_height,
        window: None,
        context: None,
        surface: None,
    };

    event_loop
        .run_app(&mut app)
        .map_err(|e| format!("Event loop error: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_color_valid() {
        let red = parse_color("red").unwrap();
        assert!((red.red() - 1.0).abs() < f32::EPSILON);
        assert!(red.green().abs() < f32::EPSILON);
        assert!(red.blue().abs() < f32::EPSILON);

        let green = parse_color("green").unwrap();
        assert!(green.red().abs() < f32::EPSILON);
        assert!((green.green() - 1.0).abs() < f32::EPSILON);

        let blue = parse_color("blue").unwrap();
        assert!((blue.blue() - 1.0).abs() < f32::EPSILON);

        let yellow = parse_color("yellow").unwrap();
        assert!((yellow.red() - 1.0).abs() < f32::EPSILON);
        assert!((yellow.green() - 1.0).abs() < f32::EPSILON);

        let white = parse_color("white").unwrap();
        assert!((white.red() - 1.0).abs() < f32::EPSILON);
        assert!((white.green() - 1.0).abs() < f32::EPSILON);
        assert!((white.blue() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_parse_color_case_insensitive() {
        assert!(parse_color("Red").is_ok());
        assert!(parse_color("RED").is_ok());
        assert!(parse_color("Blue").is_ok());
    }

    #[test]
    fn test_parse_color_invalid() {
        assert!(parse_color("purple").is_err());
        assert!(parse_color("").is_err());
        assert!(parse_color("unknown").is_err());
    }
}
