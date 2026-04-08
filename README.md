# coordinater

A desktop automation CLI for Linux, inspired by Python's PyAutoGUI. Control mouse, keyboard, capture screenshots, find images on screen, and draw overlays — all from the terminal.

## Features

- **Multi-monitor support** — target any monitor with `--monitor <id>`
- **Mouse control** — move, click, double-click, right-click, drag
- **Keyboard control** — key press, hotkeys, type text
- **Screenshots** — capture any monitor to a file
- **Template matching** — find an image on screen and get its coordinates
- **Screen overlay** — draw lines, rectangles, and circles on screen as a transparent overlay
- **Coordinate validation** — all coordinates are validated against monitor bounds before execution
- **Always logged** — every action prints what it did and where

## Requirements

### System

- **Linux** (X11 or Wayland)
- **Rust** 1.85+ (edition 2024)

### Linux dependencies

On Debian/Ubuntu:

```bash
sudo apt install libx11-dev libxcb1-dev libxcb-randr0-dev libxcb-shm0-dev libxcb-xfixes0-dev libxcb-shape0-dev libxkbcommon-dev
```

On Fedora:

```bash
sudo dnf install libX11-devel libxcb-devel libxkbcommon-devel
```

On Arch:

```bash
sudo pacman -S libx11 libxcb libxkbcommon
```

## Installation

```bash
git clone https://github.com/your-user/coordinater.git
cd coordinater
cargo build --release
```

The binary will be at `target/release/coordinater`.

## Usage

All coordinates are **relative to the selected monitor** (0,0 = top-left corner). The default monitor is the primary one.

### List monitors

```bash
coordinater monitors
# [coordinater] monitor 65: 1920x1080 (primary)
# [coordinater] monitor 515: 2560x1080
```

### Screenshot

```bash
coordinater screenshot -o screenshot.png
coordinater screenshot -o screenshot.png --monitor 515
```

### Mouse

```bash
coordinater move 500 300
coordinater click 500 300
coordinater doubleclick 500 300
coordinater rightclick 500 300
coordinater drag 100 100 400 400
```

### Scroll

```bash
coordinater scroll 5      # scroll up
coordinater scroll -3     # scroll down
```

### Keyboard

```bash
coordinater key enter
coordinater key A              # sends uppercase A
coordinater hotkey ctrl c      # press ctrl+c
coordinater hotkey alt tab     # press alt+tab
coordinater type "hello world"
```

Supported key names: `enter`, `tab`, `space`, `backspace`, `delete`, `escape`/`esc`, `up`, `down`, `left`, `right`, `home`, `end`, `pageup`, `pagedown`, `ctrl`/`control`, `alt`, `shift`, `meta`/`super`/`win`/`cmd`, `f1`–`f12`, and any single character.

### Locate image on screen

Find a template image on the current screen and return its center coordinates:

```bash
coordinater locate icon.png
# [coordinater] locate found icon.png at x=450,y=230 on monitor 65 (1920x1080)

coordinater locate button.png --threshold 0.9
coordinater locate icon.png --monitor 515
```

The `--threshold` flag (0.0–1.0, default 0.8) controls match sensitivity. Higher = stricter matching.

Returns exit code 1 if the image is not found.

### Draw overlay

Draw shapes on screen as a transparent, click-through overlay:

```bash
coordinater draw line 0 0 500 500 --color red --duration 3
coordinater draw rect 100 100 300 200 --color blue --duration 5
coordinater draw circle 500 500 50 --color green
```

Supported shapes:
- `line <x1> <y1> <x2> <y2>`
- `rect <x> <y> <width> <height>`
- `circle <x> <y> <radius>`

Colors: `red`, `green`, `blue`, `yellow`, `white` (default: `red`)

Duration: seconds the overlay stays visible (default: `3`)

### Target a specific monitor

Use the global `--monitor` flag with any command:

```bash
coordinater click 200 300 --monitor 515
coordinater screenshot -o mon2.png --monitor 515
coordinater draw circle 100 100 50 --monitor 515
```

## How it works

- **Coordinates** are always relative to the selected monitor. The CLI converts them to absolute desktop coordinates internally.
- **Validation** happens before any action. If coordinates are out of bounds, the command fails with a clear error and exit code 1.
- **Logging** is always on. Every action prints to stdout in the format `[coordinater] <action> on monitor <id> (<width>x<height>)`. Errors go to stderr.

## Architecture

```
src/
  main.rs        — CLI dispatcher
  cli.rs         — subcommand definitions (clap)
  monitor.rs     — monitor detection, coordinate validation
  events.rs      — mouse/keyboard control (enigo)
  screenshot.rs  — screen capture (xcap)
  locate.rs      — template matching
  overlay.rs     — transparent overlay (winit + tiny-skia)
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| [clap](https://crates.io/crates/clap) | CLI argument parsing |
| [enigo](https://crates.io/crates/enigo) | Mouse and keyboard control |
| [xcap](https://crates.io/crates/xcap) | Screen capture |
| [image](https://crates.io/crates/image) | Image loading and manipulation |
| [winit](https://crates.io/crates/winit) | Window creation for overlay |
| [tiny-skia](https://crates.io/crates/tiny-skia) | 2D rendering for overlay shapes |
| [softbuffer](https://crates.io/crates/softbuffer) | Pixel buffer display |

## License

MIT
