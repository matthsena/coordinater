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
    let monitors = MonitorInfo::all()?;

    match cli.command {
        Command::Monitors => {
            for m in &monitors {
                if m.is_primary {
                    println!(
                        "[coordinater] monitor {}: {}x{} (primary)",
                        m.id, m.width, m.height
                    );
                } else {
                    println!(
                        "[coordinater] monitor {}: {}x{}",
                        m.id, m.width, m.height
                    );
                }
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
            let (abs_x, abs_y) = monitor.to_absolute(x, y)?;
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
            let (abs_x, abs_y) = monitor.to_absolute(x, y)?;
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
            let (abs_x, abs_y) = monitor.to_absolute(x, y)?;
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
            let (abs_x, abs_y) = monitor.to_absolute(x, y)?;
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
            let (abs_x1, abs_y1) = monitor.to_absolute(x1, y1)?;
            let (abs_x2, abs_y2) = monitor.to_absolute(x2, y2)?;
            let mut runner = EventRunner::new()?;
            runner.drag(abs_x1, abs_y1, abs_x2, abs_y2)?;
            println!(
                "[coordinater] drag from x={},y={} to x={},y={} on monitor {} ({}x{})",
                x1, y1, x2, y2, monitor.id, monitor.width, monitor.height
            );
            Ok(())
        }

        Command::Scroll { amount } => {
            if amount == 0 {
                return Err("scroll amount must not be zero".to_string());
            }
            let monitor = MonitorInfo::resolve(&monitors, cli.monitor)?;
            let mut runner = EventRunner::new()?;
            runner.scroll(amount)?;
            let direction = if amount > 0 { "up" } else { "down" };
            let abs_amount = amount.unsigned_abs();
            println!(
                "[coordinater] scroll {} by {} on monitor {} ({}x{})",
                direction, abs_amount, monitor.id, monitor.width, monitor.height
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
            if !(0.0..=1.0).contains(&threshold) {
                return Err(format!("threshold must be between 0.0 and 1.0 (got {})", threshold));
            }
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
            if duration == 0 {
                return Err("draw duration must be greater than 0".to_string());
            }
            let monitor = MonitorInfo::resolve(&monitors, cli.monitor)?;
            let parsed_color = parse_color(&color)?;

            let (overlay_shape, shape_desc) = match shape {
                DrawShape::Line { x1, y1, x2, y2 } => {
                    monitor.validate_coords(x1, y1)?;
                    monitor.validate_coords(x2, y2)?;
                    (
                        Shape::Line {
                            x1: x1 as f32,
                            y1: y1 as f32,
                            x2: x2 as f32,
                            y2: y2 as f32,
                        },
                        format!("line from ({},{}) to ({},{})", x1, y1, x2, y2),
                    )
                }
                DrawShape::Rect {
                    x,
                    y,
                    width,
                    height,
                } => {
                    if width == 0 || height == 0 {
                        return Err("rect width and height must be greater than 0".to_string());
                    }
                    monitor.validate_coords(x, y)?;
                    let x2 = i32::try_from(width).ok()
                        .and_then(|w| x.checked_add(w - 1))
                        .ok_or_else(|| format!("rect dimensions overflow for monitor {} ({}x{})", monitor.id, monitor.width, monitor.height))?;
                    let y2 = i32::try_from(height).ok()
                        .and_then(|h| y.checked_add(h - 1))
                        .ok_or_else(|| format!("rect dimensions overflow for monitor {} ({}x{})", monitor.id, monitor.width, monitor.height))?;
                    monitor.validate_coords(x2, y2)?;
                    (
                        Shape::Rect {
                            x: x as f32,
                            y: y as f32,
                            width: width as f32,
                            height: height as f32,
                        },
                        format!("rect at ({},{}) size {}x{}", x, y, width, height),
                    )
                }
                DrawShape::Circle { x, y, radius } => {
                    if radius == 0 {
                        return Err("circle radius must be greater than 0".to_string());
                    }
                    monitor.validate_coords(x, y)?;
                    let r = i32::try_from(radius).ok()
                        .ok_or_else(|| format!("circle radius overflow for monitor {} ({}x{})", monitor.id, monitor.width, monitor.height))?;
                    monitor.validate_coords(x.saturating_sub(r), y.saturating_sub(r))?;
                    monitor.validate_coords(x.saturating_add(r), y.saturating_add(r))?;
                    (
                        Shape::Circle {
                            x: x as f32,
                            y: y as f32,
                            radius: radius as f32,
                        },
                        format!("circle at ({},{}) radius {}", x, y, radius),
                    )
                }
            };

            overlay::show_overlay(
                overlay_shape,
                parsed_color,
                duration as f64,
                monitor.x,
                monitor.y,
                monitor.width,
                monitor.height,
            )?;

            println!(
                "[coordinater] draw {} on monitor {} ({}x{}) [duration: {}s]",
                shape_desc, monitor.id, monitor.width, monitor.height, duration
            );
            Ok(())
        }
    }
}
