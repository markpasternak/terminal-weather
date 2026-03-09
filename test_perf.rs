use ratatui::{buffer::{Buffer, Cell}, layout::Rect, style::{Color, Style}};

fn main() {
    let mut buf = Buffer::empty(Rect::new(0, 0, 100, 100));
    let area = Rect::new(0, 0, 100, 100);
    let flash_bg = Color::Red;
    let spaces = "                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                ";

    // Warmup
    for _ in 0..100 {
        for y in area.top()..area.bottom() {
            let mut x = area.left();
            let right = area.right();
            let style = Style::default().bg(flash_bg);
            while x < right {
                let len = (right - x).min(spaces.len() as u16);
                buf.set_string(x, y, &spaces[0..len as usize], style);
                x += len;
            }
        }
    }

    // Method 1: set_string
    let start = std::time::Instant::now();
    for _ in 0..10000 {
        for y in area.top()..area.bottom() {
            let mut x = area.left();
            let right = area.right();
            let style = Style::default().bg(flash_bg);
            while x < right {
                let len = (right - x).min(spaces.len() as u16);
                buf.set_string(x, y, &spaces[0..len as usize], style);
                x += len;
            }
        }
    }
    println!("set_string: {:?}", start.elapsed());

    // Method 2: loop inside loop cell mut property assign
    let start = std::time::Instant::now();
    for _ in 0..10000 {
        let area = Rect::new(0, 0, 100, 100);
        let flash_bg = Color::Red;
        let style = Style::default().bg(flash_bg);

        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.set_char(' ');
                    cell.set_style(style);
                }
            }
        }
    }
    println!("cell_mut set_char: {:?}", start.elapsed());

    // Method 3: clone cell
    let start = std::time::Instant::now();
    for _ in 0..10000 {
        let area = Rect::new(0, 0, 100, 100);
        let flash_bg = Color::Red;
        let mut empty_cell = Cell::default();
        empty_cell.set_char(' ').set_bg(flash_bg);

        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                if let Some(cell) = buf.cell_mut((x, y)) {
                    *cell = empty_cell.clone();
                }
            }
        }
    }
    println!("cell copy clone: {:?}", start.elapsed());
}
