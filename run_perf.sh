#!/bin/bash
cat << 'RS' > test_perf.rs
use ratatui::{buffer::Buffer, layout::Rect, style::{Color, Style}};

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

    // Method 2: cell_mut iteration
    let start = std::time::Instant::now();
    for _ in 0..10000 {
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.set_symbol(" ");
                    cell.set_bg(flash_bg);
                }
            }
        }
    }
    println!("cell_mut: {:?}", start.elapsed());
}
RS
rustc --edition 2021 --extern ratatui=target/debug/deps/libratatui-*.rlib -L dependency=target/debug/deps test_perf.rs -O
./test_perf
