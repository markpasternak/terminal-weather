use crate::domain::weather::{WeatherCategory, weather_code_to_category};

pub(super) fn arc_glyph(x: usize, y: usize, width: usize, top: usize, mid_x: usize) -> char {
    if y == top || (y == top + 1 && (mid_x.wrapping_sub(x)) <= 2) {
        return '─';
    }
    if x <= width / 6 || x >= width * 5 / 6 {
        return '·';
    }
    if x <= width / 3 {
        return '╭';
    }
    if x >= width * 2 / 3 {
        return '╮';
    }
    '·'
}

pub(super) fn center_symbol(is_day: bool, large: bool, moon_symbol: char) -> char {
    if is_day {
        if large { '☀' } else { '◉' }
    } else {
        moon_symbol
    }
}

pub(super) fn symbol_for_code(code: u8) -> char {
    match weather_code_to_category(code) {
        WeatherCategory::Clear => 'o',
        WeatherCategory::Cloudy => '~',
        WeatherCategory::Rain => '/',
        WeatherCategory::Snow => '*',
        WeatherCategory::Fog => '=',
        WeatherCategory::Thunder => '!',
        WeatherCategory::Unknown => '?',
    }
}

pub(super) fn precip_symbol(mm: Option<f32>) -> char {
    let Some(mm) = mm else {
        return '·';
    };
    if mm >= 2.5 {
        '█'
    } else if mm >= 1.0 {
        '▓'
    } else if mm >= 0.2 {
        '▒'
    } else if mm > 0.0 {
        '░'
    } else {
        '·'
    }
}
