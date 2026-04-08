use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "coordinater", about = "Desktop automation CLI", version)]
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
