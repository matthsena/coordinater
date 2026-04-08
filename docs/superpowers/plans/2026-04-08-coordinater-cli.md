# coordinater CLI Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a PyAutoGUI-inspired desktop automation CLI in Rust with multi-monitor support, screenshot, input control, template matching, and screen overlay.

**Architecture:** Single binary CLI using `clap` derive for subcommands. Each subcommand resolves a target monitor, validates coordinates against its bounds, executes the action via dedicated modules, and logs the result. Coordinates are always relative to the selected monitor.

**Tech Stack:** Rust 2024 edition, clap (CLI), enigo (input), xcap (capture), image/imageproc (image ops), winit + tiny-skia + softbuffer (overlay)

---

## File Structure

```
src/
  main.rs          -- entry point: parse CLI, resolve monitor, dispatch to handler
  cli.rs           -- clap derive: Cli struct with subcommands enum
  monitor.rs       -- MonitorInfo with position, bounds validation, lookup by id
  events.rs        -- mouse/keyboard actions via enigo (refactored from current)
  screenshot.rs    -- capture a specific monitor's screen via xcap
  locate.rs        -- template matching: find an image on screen
  overlay.rs       -- transparent overlay window with winit + tiny-skia + softbuffer
```

---

### Task 1: Add dependencies and create cli.rs

**Files:**
- Modify: `Cargo.toml`
- Create: `src/cli.rs`

- [ ] **Step 1: Add clap and softbuffer to Cargo.toml**

Add `clap` with derive feature and `softbuffer` to `[dependencies]` in `Cargo.toml`:

```toml
[dependencies]
clap = { version = "4", features = ["derive"] }
enigo = "0.6.1"
fs_extra = "1.3.0"
image = "0.25.10"
imageproc = "0.26.1"
softbuffer = "0.4"
tiny-skia = "0.11"
tokio = "1.50.0"
winit = "0.30"
xcap = "0.9.2"
```

- [ ] **Step 2: Create src/cli.rs with all subcommand definitions**

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "coordinater", about = "Desktop automation CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    /// Target monitor ID (default: primary)
    #[arg(long, global = true)]
    pub monitor: Option<u32>,
}

#[derive(Subcommand)]
pub enum Command {
    /// List all available monitors
    Monitors,

    /// Take a screenshot of a monitor
    Screenshot {
        /// Output file path
        #[arg(short)]
        o: String,
    },

    /// Move the mouse to x,y
    Move {
        x: i32,
        y: i32,
    },

    /// Left click at x,y
    Click {
        x: i32,
        y: i32,
    },

    /// Double click at x,y
    Doubleclick {
        x: i32,
        y: i32,
    },

    /// Right click at x,y
    Rightclick {
        x: i32,
        y: i32,
    },

    /// Drag from (x1,y1) to (x2,y2)
    Drag {
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
    },

    /// Scroll (positive=up, negative=down)
    Scroll {
        amount: i32,
    },

    /// Press a single key
    Key {
        key: String,
    },

    /// Press a key combination (e.g. ctrl c)
    Hotkey {
        keys: Vec<String>,
    },

    /// Type a string
    Type {
        text: String,
    },

    /// Find an image on screen, return coordinates
    Locate {
        image: String,

        /// Match sensitivity 0.0-1.0
        #[arg(long, default_value_t = 0.8)]
        threshold: f64,
    },

    /// Draw an overlay shape on screen
    Draw {
        #[command(subcommand)]
        shape: DrawShape,

        /// Overlay color
        #[arg(long, default_value = "red", global = true)]
        color: String,

        /// Duration in seconds
        #[arg(long, default_value_t = 3, global = true)]
        duration: u64,
    },
}

#[derive(Subcommand)]
pub enum DrawShape {
    /// Draw a line from (x1,y1) to (x2,y2)
    Line {
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
    },
    /// Draw a rectangle at (x,y) with width and height
    Rect {
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    },
    /// Draw a circle at (x,y) with radius
    Circle {
        x: i32,
        y: i32,
        radius: u32,
    },
}

```

- [ ] **Step 3: Verify it compiles**

Run: `cargo check`
Expected: compiles with warnings about unused imports (modules not wired yet)

- [ ] **Step 4: Commit**

```bash
git add Cargo.toml Cargo.lock src/cli.rs
git commit -m "feat: add clap CLI with all subcommand definitions"
```

---

### Task 2: Refactor monitor.rs — position, validation, lookup

**Files:**
- Modify: `src/monitor.rs`

- [ ] **Step 1: Write tests for coordinate validation and monitor lookup**

Add at the bottom of `src/monitor.rs`:

```rust
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
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --lib monitor`
Expected: FAIL — `validate_coords`, `to_absolute`, `find_by_id`, `find_primary` not found

- [ ] **Step 3: Implement the full monitor.rs**

Replace the entire content of `src/monitor.rs` with:

```rust
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
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --lib monitor`
Expected: all 5 tests pass

- [ ] **Step 5: Commit**

```bash
git add src/monitor.rs
git commit -m "feat: add coordinate validation and monitor lookup"
```

---

### Task 3: Refactor events.rs — coordinate-aware input

**Files:**
- Modify: `src/events.rs`

- [ ] **Step 1: Refactor events.rs to support absolute coordinate conversion**

Replace the entire content of `src/events.rs` with:

```rust
use enigo::{Axis, Button, Coordinate, Direction, Enigo, Key, Keyboard, Mouse, Settings};

pub struct EventRunner {
    enigo: Enigo,
}

impl EventRunner {
    pub fn new() -> Result<Self, String> {
        let enigo = Enigo::new(&Settings::default())
            .map_err(|e| format!("failed to initialize input controller: {}", e))?;
        Ok(Self { enigo })
    }

    pub fn move_mouse(&mut self, abs_x: i32, abs_y: i32) -> Result<(), String> {
        self.enigo
            .move_mouse(abs_x, abs_y, Coordinate::Abs)
            .map_err(|e| format!("failed to move mouse: {}", e))
    }

    pub fn click(&mut self, abs_x: i32, abs_y: i32) -> Result<(), String> {
        self.move_mouse(abs_x, abs_y)?;
        self.enigo
            .button(Button::Left, Direction::Click)
            .map_err(|e| format!("failed to click: {}", e))
    }

    pub fn double_click(&mut self, abs_x: i32, abs_y: i32) -> Result<(), String> {
        self.move_mouse(abs_x, abs_y)?;
        self.enigo
            .button(Button::Left, Direction::Click)
            .map_err(|e| format!("failed to double click: {}", e))?;
        self.enigo
            .button(Button::Left, Direction::Click)
            .map_err(|e| format!("failed to double click: {}", e))
    }

    pub fn right_click(&mut self, abs_x: i32, abs_y: i32) -> Result<(), String> {
        self.move_mouse(abs_x, abs_y)?;
        self.enigo
            .button(Button::Right, Direction::Click)
            .map_err(|e| format!("failed to right click: {}", e))
    }

    pub fn drag(&mut self, from_x: i32, from_y: i32, to_x: i32, to_y: i32) -> Result<(), String> {
        self.move_mouse(from_x, from_y)?;
        self.enigo
            .button(Button::Left, Direction::Press)
            .map_err(|e| format!("failed to start drag: {}", e))?;
        self.move_mouse(to_x, to_y)?;
        self.enigo
            .button(Button::Left, Direction::Release)
            .map_err(|e| format!("failed to end drag: {}", e))
    }

    pub fn scroll(&mut self, amount: i32) -> Result<(), String> {
        self.enigo
            .scroll(amount, Axis::Vertical)
            .map_err(|e| format!("failed to scroll: {}", e))
    }

    pub fn key_press(&mut self, key: Key) -> Result<(), String> {
        self.enigo
            .key(key, Direction::Click)
            .map_err(|e| format!("failed to press key: {}", e))
    }

    pub fn hotkey(&mut self, keys: &[Key]) -> Result<(), String> {
        for key in keys {
            self.enigo
                .key(*key, Direction::Press)
                .map_err(|e| format!("failed to press key: {}", e))?;
        }
        for key in keys.iter().rev() {
            self.enigo
                .key(*key, Direction::Release)
                .map_err(|e| format!("failed to release key: {}", e))?;
        }
        Ok(())
    }

    pub fn type_text(&mut self, text: &str) -> Result<(), String> {
        self.enigo
            .text(text)
            .map_err(|e| format!("failed to type text: {}", e))
    }
}

pub fn parse_key(name: &str) -> Result<Key, String> {
    match name.to_lowercase().as_str() {
        "enter" | "return" => Ok(Key::Return),
        "tab" => Ok(Key::Tab),
        "space" => Ok(Key::Space),
        "backspace" => Ok(Key::Backspace),
        "delete" => Ok(Key::Delete),
        "escape" | "esc" => Ok(Key::Escape),
        "up" => Ok(Key::UpArrow),
        "down" => Ok(Key::DownArrow),
        "left" => Ok(Key::LeftArrow),
        "right" => Ok(Key::RightArrow),
        "home" => Ok(Key::Home),
        "end" => Ok(Key::End),
        "pageup" => Ok(Key::PageUp),
        "pagedown" => Ok(Key::PageDown),
        "ctrl" | "control" => Ok(Key::Control),
        "alt" => Ok(Key::Alt),
        "shift" => Ok(Key::Shift),
        "meta" | "super" | "win" | "cmd" => Ok(Key::Meta),
        "f1" => Ok(Key::F1),
        "f2" => Ok(Key::F2),
        "f3" => Ok(Key::F3),
        "f4" => Ok(Key::F4),
        "f5" => Ok(Key::F5),
        "f6" => Ok(Key::F6),
        "f7" => Ok(Key::F7),
        "f8" => Ok(Key::F8),
        "f9" => Ok(Key::F9),
        "f10" => Ok(Key::F10),
        "f11" => Ok(Key::F11),
        "f12" => Ok(Key::F12),
        s if s.len() == 1 => Ok(Key::Unicode(s.chars().next().unwrap())),
        other => Err(format!("unknown key: {}", other)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_key_named() {
        assert!(matches!(parse_key("enter"), Ok(Key::Return)));
        assert!(matches!(parse_key("ENTER"), Ok(Key::Return)));
        assert!(matches!(parse_key("ctrl"), Ok(Key::Control)));
        assert!(matches!(parse_key("escape"), Ok(Key::Escape)));
        assert!(matches!(parse_key("esc"), Ok(Key::Escape)));
        assert!(matches!(parse_key("f1"), Ok(Key::F1)));
    }

    #[test]
    fn test_parse_key_single_char() {
        assert!(matches!(parse_key("a"), Ok(Key::Unicode('a'))));
        assert!(matches!(parse_key("z"), Ok(Key::Unicode('z'))));
        assert!(matches!(parse_key("1"), Ok(Key::Unicode('1'))));
    }

    #[test]
    fn test_parse_key_unknown() {
        assert!(parse_key("nonexistent").is_err());
        assert!(parse_key("ab").is_err());
    }
}
```

- [ ] **Step 2: Run tests to verify**

Run: `cargo test --lib events`
Expected: all 3 tests pass

- [ ] **Step 3: Commit**

```bash
git add src/events.rs
git commit -m "refactor: events module with coordinate-aware input and key parsing"
```

---

### Task 4: Create screenshot.rs

**Files:**
- Create: `src/screenshot.rs`

- [ ] **Step 1: Create src/screenshot.rs**

```rust
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
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo check`
Expected: compiles (with warnings about unused module — not wired yet)

- [ ] **Step 3: Commit**

```bash
git add src/screenshot.rs
git commit -m "feat: screenshot capture module"
```

---

### Task 5: Create locate.rs — template matching

**Files:**
- Create: `src/locate.rs`

- [ ] **Step 1: Create src/locate.rs**

```rust
use image::{DynamicImage, GenericImageView, Rgba};
use xcap::Monitor;

pub struct LocateResult {
    pub x: i32,
    pub y: i32,
}

pub fn find_on_screen(
    monitor_id: u32,
    template_path: &str,
    threshold: f64,
) -> Result<LocateResult, String> {
    let monitors = Monitor::all()
        .map_err(|e| format!("failed to list monitors: {}", e))?;

    let monitor = monitors
        .into_iter()
        .find(|m| m.id().unwrap_or(0) == monitor_id)
        .ok_or_else(|| format!("monitor {} not found", monitor_id))?;

    let screenshot = monitor
        .capture_image()
        .map_err(|e| format!("failed to capture screen: {}", e))?;

    let screen = DynamicImage::ImageRgba8(screenshot);
    let template = image::open(template_path)
        .map_err(|e| format!("failed to open template image: {}", e))?;

    find_template(&screen, &template, threshold)
}

fn find_template(
    screen: &DynamicImage,
    template: &DynamicImage,
    threshold: f64,
) -> Result<LocateResult, String> {
    let (sw, sh) = screen.dimensions();
    let (tw, th) = template.dimensions();

    if tw > sw || th > sh {
        return Err("template is larger than screen".to_string());
    }

    let mut best_score = f64::MAX;
    let mut best_x = 0u32;
    let mut best_y = 0u32;

    let max_possible_diff = (tw * th) as f64 * 255.0 * 3.0;

    for y in 0..=(sh - th) {
        for x in 0..=(sw - tw) {
            let mut diff_sum: f64 = 0.0;

            for ty in 0..th {
                for tx in 0..tw {
                    let Rgba(sp) = screen.get_pixel(x + tx, y + ty);
                    let Rgba(tp) = template.get_pixel(tx, ty);

                    diff_sum += (sp[0] as f64 - tp[0] as f64).abs();
                    diff_sum += (sp[1] as f64 - tp[1] as f64).abs();
                    diff_sum += (sp[2] as f64 - tp[2] as f64).abs();
                }
            }

            let similarity = 1.0 - (diff_sum / max_possible_diff);

            if similarity > threshold && diff_sum < best_score {
                best_score = diff_sum;
                best_x = x;
                best_y = y;
            }
        }
    }

    if best_score == f64::MAX {
        return Err(format!(
            "could not find image on screen (threshold: {})",
            threshold
        ));
    }

    Ok(LocateResult {
        x: (best_x + tw / 2) as i32,
        y: (best_y + th / 2) as i32,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{DynamicImage, RgbaImage, Rgba};

    #[test]
    fn test_find_template_exact_match() {
        let mut screen = RgbaImage::new(100, 100);
        for y in 0..100 {
            for x in 0..100 {
                screen.put_pixel(x, y, Rgba([0, 0, 0, 255]));
            }
        }
        // Place a 10x10 red square at (30, 40)
        for y in 40..50 {
            for x in 30..40 {
                screen.put_pixel(x, y, Rgba([255, 0, 0, 255]));
            }
        }

        let mut template = RgbaImage::new(10, 10);
        for y in 0..10 {
            for x in 0..10 {
                template.put_pixel(x, y, Rgba([255, 0, 0, 255]));
            }
        }

        let screen = DynamicImage::ImageRgba8(screen);
        let template = DynamicImage::ImageRgba8(template);

        let result = find_template(&screen, &template, 0.8).unwrap();
        assert_eq!(result.x, 35); // center of 30..40
        assert_eq!(result.y, 45); // center of 40..50
    }

    #[test]
    fn test_find_template_not_found() {
        let screen = DynamicImage::ImageRgba8(RgbaImage::new(100, 100));
        let mut template = RgbaImage::new(10, 10);
        for y in 0..10 {
            for x in 0..10 {
                template.put_pixel(x, y, Rgba([255, 0, 0, 255]));
            }
        }
        let template = DynamicImage::ImageRgba8(template);

        let result = find_template(&screen, &template, 0.8);
        assert!(result.is_err());
    }

    #[test]
    fn test_template_larger_than_screen() {
        let screen = DynamicImage::ImageRgba8(RgbaImage::new(5, 5));
        let template = DynamicImage::ImageRgba8(RgbaImage::new(10, 10));

        let result = find_template(&screen, &template, 0.8);
        assert!(result.is_err());
    }
}
```

- [ ] **Step 2: Run tests to verify**

Run: `cargo test --lib locate`
Expected: all 3 tests pass

- [ ] **Step 3: Commit**

```bash
git add src/locate.rs
git commit -m "feat: template matching locate module"
```

---

### Task 6: Create overlay.rs — transparent window drawing

**Files:**
- Create: `src/overlay.rs`

- [ ] **Step 1: Create src/overlay.rs**

```rust
use std::num::NonZeroU32;
use std::time::{Duration, Instant};

use softbuffer::Surface;
use tiny_skia::{Color, Paint, PathBuilder, Pixmap, Stroke, Transform};
use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId, WindowLevel};

pub enum Shape {
    Line { x1: i32, y1: i32, x2: i32, y2: i32 },
    Rect { x: i32, y: i32, width: u32, height: u32 },
    Circle { x: i32, y: i32, radius: u32 },
}

pub fn parse_color(name: &str) -> Result<Color, String> {
    match name.to_lowercase().as_str() {
        "red" => Ok(Color::from_rgba8(255, 0, 0, 255)),
        "green" => Ok(Color::from_rgba8(0, 255, 0, 255)),
        "blue" => Ok(Color::from_rgba8(0, 0, 255, 255)),
        "yellow" => Ok(Color::from_rgba8(255, 255, 0, 255)),
        "white" => Ok(Color::from_rgba8(255, 255, 255, 255)),
        other => Err(format!("unknown color: {} (available: red, green, blue, yellow, white)", other)),
    }
}

struct OverlayApp {
    shape: Shape,
    color: Color,
    duration: Duration,
    monitor_x: i32,
    monitor_y: i32,
    monitor_width: u32,
    monitor_height: u32,
    window: Option<Window>,
    surface: Option<Surface<&'static Window, &'static Window>>,
    start: Option<Instant>,
}

pub fn show_overlay(
    shape: Shape,
    color: Color,
    duration_secs: u64,
    monitor_x: i32,
    monitor_y: i32,
    monitor_width: u32,
    monitor_height: u32,
) -> Result<(), String> {
    let event_loop = EventLoop::new()
        .map_err(|e| format!("failed to create event loop: {}", e))?;

    let mut app = OverlayApp {
        shape,
        color,
        duration: Duration::from_secs(duration_secs),
        monitor_x,
        monitor_y,
        monitor_width,
        monitor_height,
        window: None,
        surface: None,
        start: None,
    };

    event_loop
        .run_app(&mut app)
        .map_err(|e| format!("overlay error: {}", e))
}

impl ApplicationHandler for OverlayApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let attrs = Window::default_attributes()
            .with_title("coordinater overlay")
            .with_inner_size(PhysicalSize::new(self.monitor_width, self.monitor_height))
            .with_position(PhysicalPosition::new(self.monitor_x, self.monitor_y))
            .with_transparent(true)
            .with_decorations(false)
            .with_window_level(WindowLevel::AlwaysOnTop);

        match event_loop.create_window(attrs) {
            Ok(window) => {
                self.start = Some(Instant::now());
                self.window = Some(window);
            }
            Err(e) => {
                eprintln!("[coordinater] error: failed to create overlay window: {}", e);
                event_loop.exit();
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if let Some(start) = self.start {
                    if start.elapsed() >= self.duration {
                        event_loop.exit();
                        return;
                    }
                }

                let Some(window) = self.window.as_ref() else { return };
                let size = window.inner_size();
                let width = size.width;
                let height = size.height;

                let mut pixmap = Pixmap::new(width, height)
                    .expect("failed to create pixmap");

                let mut paint = Paint::default();
                paint.set_color(self.color);
                paint.anti_alias = true;

                let mut stroke = Stroke::default();
                stroke.width = 3.0;

                match &self.shape {
                    Shape::Line { x1, y1, x2, y2 } => {
                        let mut pb = PathBuilder::new();
                        pb.move_to(*x1 as f32, *y1 as f32);
                        pb.line_to(*x2 as f32, *y2 as f32);
                        if let Some(path) = pb.finish() {
                            pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
                        }
                    }
                    Shape::Rect { x, y, width: w, height: h } => {
                        let mut pb = PathBuilder::new();
                        let x = *x as f32;
                        let y = *y as f32;
                        let w = *w as f32;
                        let h = *h as f32;
                        pb.move_to(x, y);
                        pb.line_to(x + w, y);
                        pb.line_to(x + w, y + h);
                        pb.line_to(x, y + h);
                        pb.close();
                        if let Some(path) = pb.finish() {
                            pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
                        }
                    }
                    Shape::Circle { x, y, radius } => {
                        let cx = *x as f32;
                        let cy = *y as f32;
                        let r = *radius as f32;
                        // Approximate circle with 4 cubic bezier curves
                        let k = 0.5522848;
                        let mut pb = PathBuilder::new();
                        pb.move_to(cx + r, cy);
                        pb.cubic_to(cx + r, cy + r * k, cx + r * k, cy + r, cx, cy + r);
                        pb.cubic_to(cx - r * k, cy + r, cx - r, cy + r * k, cx - r, cy);
                        pb.cubic_to(cx - r, cy - r * k, cx - r * k, cy - r, cx, cy - r);
                        pb.cubic_to(cx + r * k, cy - r, cx + r, cy - r * k, cx + r, cy);
                        pb.close();
                        if let Some(path) = pb.finish() {
                            pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
                        }
                    }
                }

                // Copy pixmap to window via softbuffer
                let context = softbuffer::Context::new(window)
                    .expect("failed to create softbuffer context");
                let mut surface = Surface::new(&context, window)
                    .expect("failed to create surface");
                surface.resize(
                    NonZeroU32::new(width).unwrap(),
                    NonZeroU32::new(height).unwrap(),
                ).expect("failed to resize surface");

                let mut buffer = surface.buffer_mut().expect("failed to get buffer");
                let pixels = pixmap.pixels();
                for (i, pixel) in pixels.iter().enumerate() {
                    let r = pixel.red() as u32;
                    let g = pixel.green() as u32;
                    let b = pixel.blue() as u32;
                    buffer[i] = (r << 16) | (g << 8) | b;
                }
                buffer.present().expect("failed to present buffer");

                // Request next redraw to check timeout
                window.request_redraw();
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_color_valid() {
        assert!(parse_color("red").is_ok());
        assert!(parse_color("RED").is_ok());
        assert!(parse_color("green").is_ok());
        assert!(parse_color("blue").is_ok());
        assert!(parse_color("yellow").is_ok());
        assert!(parse_color("white").is_ok());
    }

    #[test]
    fn test_parse_color_invalid() {
        assert!(parse_color("purple").is_err());
        assert!(parse_color("unknown").is_err());
    }
}
```

- [ ] **Step 2: Run tests to verify**

Run: `cargo test --lib overlay`
Expected: 2 tests pass

- [ ] **Step 3: Commit**

```bash
git add src/overlay.rs
git commit -m "feat: overlay module with winit + tiny-skia"
```

---

### Task 7: Wire up main.rs — dispatch all subcommands

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Replace src/main.rs with the full dispatcher**

```rust
mod cli;
mod events;
mod locate;
mod monitor;
mod overlay;
mod screenshot;

use clap::Parser;
use cli::{Cli, Command, DrawShape};
use events::{parse_key, EventRunner};
use monitor::MonitorInfo;
use overlay::{parse_color, Shape};
use std::process;

fn main() {
    if let Err(e) = run() {
        eprintln!("[coordinater] error: {}", e);
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let cli = Cli::parse();
    let monitors = MonitorInfo::all();

    match cli.command {
        Command::Monitors => {
            for m in &monitors {
                let primary = if m.is_primary { " (primary)" } else { "" };
                println!(
                    "[coordinater] monitor {}: {}x{}{}",
                    m.id, m.width, m.height, primary
                );
            }
            Ok(())
        }

        Command::Screenshot { o } => {
            let monitor = MonitorInfo::resolve(&monitors, cli.monitor)?;
            screenshot::capture(monitor.id, &o)?;
            println!(
                "[coordinater] screenshot saved to {} from monitor {} ({}x{})",
                o, monitor.id, monitor.width, monitor.height
            );
            Ok(())
        }

        Command::Move { x, y } => {
            let monitor = MonitorInfo::resolve(&monitors, cli.monitor)?;
            monitor.validate_coords(x, y)?;
            let (abs_x, abs_y) = monitor.to_absolute(x, y);
            let mut runner = EventRunner::new()?;
            runner.move_mouse(abs_x, abs_y)?;
            println!(
                "[coordinater] move to x={},y={} on monitor {} ({}x{})",
                x, y, monitor.id, monitor.width, monitor.height
            );
            Ok(())
        }

        Command::Click { x, y } => {
            let monitor = MonitorInfo::resolve(&monitors, cli.monitor)?;
            monitor.validate_coords(x, y)?;
            let (abs_x, abs_y) = monitor.to_absolute(x, y);
            let mut runner = EventRunner::new()?;
            runner.click(abs_x, abs_y)?;
            println!(
                "[coordinater] click at x={},y={} on monitor {} ({}x{})",
                x, y, monitor.id, monitor.width, monitor.height
            );
            Ok(())
        }

        Command::Doubleclick { x, y } => {
            let monitor = MonitorInfo::resolve(&monitors, cli.monitor)?;
            monitor.validate_coords(x, y)?;
            let (abs_x, abs_y) = monitor.to_absolute(x, y);
            let mut runner = EventRunner::new()?;
            runner.double_click(abs_x, abs_y)?;
            println!(
                "[coordinater] doubleclick at x={},y={} on monitor {} ({}x{})",
                x, y, monitor.id, monitor.width, monitor.height
            );
            Ok(())
        }

        Command::Rightclick { x, y } => {
            let monitor = MonitorInfo::resolve(&monitors, cli.monitor)?;
            monitor.validate_coords(x, y)?;
            let (abs_x, abs_y) = monitor.to_absolute(x, y);
            let mut runner = EventRunner::new()?;
            runner.right_click(abs_x, abs_y)?;
            println!(
                "[coordinater] rightclick at x={},y={} on monitor {} ({}x{})",
                x, y, monitor.id, monitor.width, monitor.height
            );
            Ok(())
        }

        Command::Drag { x1, y1, x2, y2 } => {
            let monitor = MonitorInfo::resolve(&monitors, cli.monitor)?;
            monitor.validate_coords(x1, y1)?;
            monitor.validate_coords(x2, y2)?;
            let (abs_x1, abs_y1) = monitor.to_absolute(x1, y1);
            let (abs_x2, abs_y2) = monitor.to_absolute(x2, y2);
            let mut runner = EventRunner::new()?;
            runner.drag(abs_x1, abs_y1, abs_x2, abs_y2)?;
            println!(
                "[coordinater] drag from x={},y={} to x={},y={} on monitor {} ({}x{})",
                x1, y1, x2, y2, monitor.id, monitor.width, monitor.height
            );
            Ok(())
        }

        Command::Scroll { amount } => {
            let monitor = MonitorInfo::resolve(&monitors, cli.monitor)?;
            let mut runner = EventRunner::new()?;
            runner.scroll(amount)?;
            let direction = if amount > 0 { "up" } else { "down" };
            println!(
                "[coordinater] scroll {} by {} on monitor {} ({}x{})",
                direction,
                amount.abs(),
                monitor.id,
                monitor.width,
                monitor.height
            );
            Ok(())
        }

        Command::Key { key } => {
            let parsed = parse_key(&key)?;
            let mut runner = EventRunner::new()?;
            runner.key_press(parsed)?;
            println!("[coordinater] key press: {}", key);
            Ok(())
        }

        Command::Hotkey { keys } => {
            let parsed: Vec<_> = keys
                .iter()
                .map(|k| parse_key(k))
                .collect::<Result<Vec<_>, _>>()?;
            let mut runner = EventRunner::new()?;
            runner.hotkey(&parsed)?;
            println!("[coordinater] hotkey: {}", keys.join("+"));
            Ok(())
        }

        Command::Type { text } => {
            let mut runner = EventRunner::new()?;
            runner.type_text(&text)?;
            println!("[coordinater] typed: \"{}\"", text);
            Ok(())
        }

        Command::Locate { image, threshold } => {
            let monitor = MonitorInfo::resolve(&monitors, cli.monitor)?;
            let result = locate::find_on_screen(monitor.id, &image, threshold)?;
            println!(
                "[coordinater] locate found {} at x={},y={} on monitor {} ({}x{})",
                image, result.x, result.y, monitor.id, monitor.width, monitor.height
            );
            Ok(())
        }

        Command::Draw {
            shape,
            color,
            duration,
        } => {
            let monitor = MonitorInfo::resolve(&monitors, cli.monitor)?;
            let parsed_color = parse_color(&color)?;

            let (overlay_shape, log_msg) = match shape {
                DrawShape::Line { x1, y1, x2, y2 } => {
                    monitor.validate_coords(x1, y1)?;
                    monitor.validate_coords(x2, y2)?;
                    (
                        Shape::Line { x1, y1, x2, y2 },
                        format!("draw line from ({},{}) to ({},{})", x1, y1, x2, y2),
                    )
                }
                DrawShape::Rect { x, y, width, height } => {
                    monitor.validate_coords(x, y)?;
                    monitor.validate_coords(x + width as i32 - 1, y + height as i32 - 1)?;
                    (
                        Shape::Rect { x, y, width, height },
                        format!("draw rect at ({},{}) size {}x{}", x, y, width, height),
                    )
                }
                DrawShape::Circle { x, y, radius } => {
                    monitor.validate_coords(x, y)?;
                    (
                        Shape::Circle { x, y, radius },
                        format!("draw circle at ({},{}) radius {}", x, y, radius),
                    )
                }
            };

            println!(
                "[coordinater] {} on monitor {} ({}x{}) [duration: {}s]",
                log_msg, monitor.id, monitor.width, monitor.height, duration
            );

            overlay::show_overlay(
                overlay_shape,
                parsed_color,
                duration,
                monitor.x,
                monitor.y,
                monitor.width,
                monitor.height,
            )
        }
    }
}
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo check`
Expected: compiles with no errors

- [ ] **Step 3: Verify basic commands work**

Run: `cargo run -- monitors`
Expected: lists available monitors

Run: `cargo run -- --help`
Expected: shows all subcommands with descriptions

- [ ] **Step 4: Commit**

```bash
git add src/main.rs
git commit -m "feat: wire up CLI dispatcher for all subcommands"
```

---

### Task 8: Smoke-test all subcommands

Manual verification of each subcommand on a real display.

- [ ] **Step 1: Test monitors**

Run: `cargo run -- monitors`
Expected: `[coordinater] monitor N: WxH (primary)` for each monitor

- [ ] **Step 2: Test screenshot**

Run: `cargo run -- screenshot -o /tmp/test-screenshot.png`
Expected: file created, log message printed

- [ ] **Step 3: Test mouse move**

Run: `cargo run -- move 100 100`
Expected: mouse moves, log printed

- [ ] **Step 4: Test click**

Run: `cargo run -- click 100 100`
Expected: click happens, log printed

- [ ] **Step 5: Test coordinate validation**

Run: `cargo run -- move 99999 99999`
Expected: error message about out of bounds

- [ ] **Step 6: Test key**

Run: `cargo run -- key space`
Expected: log printed

- [ ] **Step 7: Test hotkey**

Run: `cargo run -- hotkey ctrl c`
Expected: `[coordinater] hotkey: ctrl+c`

- [ ] **Step 8: Test type**

Run: `cargo run -- type "hello"`
Expected: text typed, log printed

- [ ] **Step 9: Test scroll**

Run: `cargo run -- scroll 3`
Expected: scrolls up, log printed

- [ ] **Step 10: Test draw**

Run: `cargo run -- draw line 100 100 500 500 --color red --duration 2`
Expected: red line overlay appears for 2 seconds

- [ ] **Step 11: Commit any fixes**

```bash
git add -A
git commit -m "fix: smoke test adjustments"
```

---

### Task 9: Run all unit tests

- [ ] **Step 1: Run the full test suite**

Run: `cargo test`
Expected: all tests pass (monitor, events, locate, overlay)

- [ ] **Step 2: Final commit**

```bash
git add -A
git commit -m "test: verify all unit tests pass"
```
