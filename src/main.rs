use screen_size::get_primary_screen_size;
use screenshots::Screen;

fn main() {
    let mut width = 0;
    let mut height = 0;
    if let Ok((w, h)) = get_primary_screen_size() {
        width = w;
        height = h;
    }

    let screens = Screen::all().unwrap();
}
