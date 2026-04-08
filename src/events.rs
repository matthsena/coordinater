use enigo::{Axis, Button, Coordinate, Direction, Enigo, Key, Keyboard, Mouse, Settings};

pub struct EventRunner {
    enigo: Enigo,
}

impl EventRunner {
    pub fn new() -> Self {
        Self {
            enigo: Enigo::new(&Settings::default()).unwrap(),
        }
    }

    pub fn scroll(&mut self, length: i32, axis: Option<Axis>) {
        let axis = axis.unwrap_or(Axis::Vertical);
        self.enigo.scroll(length, axis).unwrap();
    }

    pub fn key_event(&mut self, key: Key, direction: Direction) {
        self.enigo.key(key, direction).unwrap();
    }

    pub fn move_mouse(&mut self, x: i32, y: i32, coordinate: Option<Coordinate>) {
        let coordinate = coordinate.unwrap_or(Coordinate::Abs);
        self.enigo.move_mouse(x, y, coordinate).unwrap();
    }

    pub fn click(&mut self, button: Button, direction: Direction) {
        self.enigo.button(button, direction).unwrap();
    }
}
