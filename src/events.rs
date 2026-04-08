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
