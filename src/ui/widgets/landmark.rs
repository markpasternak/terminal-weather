#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LandmarkTint {
    Warm,
    Cool,
    Neutral,
}

#[derive(Debug, Clone)]
pub struct LandmarkScene {
    pub label: String,
    pub lines: Vec<String>,
    pub tint: LandmarkTint,
}

pub fn scene_for_location(
    location_name: &str,
    is_day: bool,
    frame_tick: u64,
    animate: bool,
    width: u16,
    height: u16,
) -> LandmarkScene {
    let compact = width < 20 || height < 8;
    let norm = normalize(location_name);
    let phase = if animate { frame_tick % 12 } else { 0 };
    let twinkle = if animate && phase % 2 == 0 { '*' } else { '.' };

    let mut scene = if norm.contains("stockholm") {
        stockholm_scene(twinkle, compact)
    } else if norm.contains("paris") {
        paris_scene(twinkle, compact)
    } else if norm.contains("new york") || norm.contains("nyc") {
        new_york_scene(twinkle, compact)
    } else if norm.contains("tokyo") {
        tokyo_scene(twinkle, compact)
    } else if norm.contains("london") {
        london_scene(twinkle, compact)
    } else if norm.contains("sydney") {
        sydney_scene(twinkle, compact)
    } else {
        city_signature_scene(
            &norm,
            is_day,
            phase,
            compact,
            width as usize,
            height as usize,
        )
    };

    scene.lines = scene
        .lines
        .into_iter()
        .map(|line| fit_line(&line, width as usize))
        .take(height as usize)
        .collect();
    scene
}

fn stockholm_scene(twinkle: char, compact: bool) -> LandmarkScene {
    let lines = if compact {
        vec![" STO CITY ".to_string(), format!("  ~{}~~~   ", twinkle)]
    } else {
        vec![
            "         /\\            ".to_string(),
            "        /  \\   ^ ^ ^   ".to_string(),
            "       /____\\  | | |   ".to_string(),
            "      | [] |___|_|_|_  ".to_string(),
            "   ___|____|  _  _  |  ".to_string(),
            "  |  _  _  | | || |||  ".to_string(),
            "  |_|_|_|_|_|_||_|||_  ".to_string(),
            format!("  ~~~~~{}~~~~~~~ ~~   ", twinkle),
        ]
    };

    LandmarkScene {
        label: "Stockholm City Hall".to_string(),
        lines,
        tint: LandmarkTint::Warm,
    }
}

fn paris_scene(twinkle: char, compact: bool) -> LandmarkScene {
    let lines = if compact {
        vec![" PARIS ".to_string(), format!("  /{}\\  ", twinkle)]
    } else {
        vec![
            "          /\\            ".to_string(),
            "         /  \\           ".to_string(),
            "        /_/\\_\\          ".to_string(),
            "       /_/  \\_\\         ".to_string(),
            "      /_/====\\_\\        ".to_string(),
            "         ||||           ".to_string(),
            "         ||||           ".to_string(),
            format!("  ~~~~~~~{}~~~~~~~      ", twinkle),
        ]
    };

    LandmarkScene {
        label: "Eiffel Tower".to_string(),
        lines,
        tint: LandmarkTint::Warm,
    }
}

fn new_york_scene(twinkle: char, compact: bool) -> LandmarkScene {
    let window = if twinkle == '*' { 'o' } else { '.' };
    let lines = if compact {
        vec![
            " NYC ".to_string(),
            format!(" {}{}{}{}{} ", window, window, window, window, window),
        ]
    } else {
        vec![
            "   |-|   |-|    /\\      ".to_string(),
            "   | |___| |   /  \\     ".to_string(),
            format!(" __|{}|_{}|_|__/[]_\\___  ", window, window),
            format!("| [] {}  []  []  {} [] | ", window, window),
            "|_[]__[]__[]__[]__[]__| ".to_string(),
            "   ||    ||    ||    || ".to_string(),
            "~~~~~~~~~~~~~~~~~~~~~~~ ".to_string(),
        ]
    };

    LandmarkScene {
        label: "NYC Skyline".to_string(),
        lines,
        tint: LandmarkTint::Cool,
    }
}

fn tokyo_scene(twinkle: char, compact: bool) -> LandmarkScene {
    let lines = if compact {
        vec![" TOKYO ".to_string(), format!("  /{}\\   ", twinkle)]
    } else {
        vec![
            "         /\\             ".to_string(),
            "        /##\\            ".to_string(),
            "       /####\\           ".to_string(),
            "      /######\\          ".to_string(),
            "         ||             ".to_string(),
            "       __||__           ".to_string(),
            "      /__||__\\          ".to_string(),
            format!("  ~~~~~~{}~~~~~~~       ", twinkle),
        ]
    };

    LandmarkScene {
        label: "Tokyo Tower".to_string(),
        lines,
        tint: LandmarkTint::Cool,
    }
}

fn london_scene(twinkle: char, compact: bool) -> LandmarkScene {
    let lines = if compact {
        vec![" LONDON ".to_string(), format!("  [{}]   ", twinkle)]
    } else {
        vec![
            "         []             ".to_string(),
            "         ||             ".to_string(),
            "      ___||___          ".to_string(),
            "     |  __   |          ".to_string(),
            "     | |[]|  |          ".to_string(),
            "     | |  |  |__        ".to_string(),
            "   __|_|__|__|__|       ".to_string(),
            format!("  ~~~~~{}~~~~~~~~~      ", twinkle),
        ]
    };

    LandmarkScene {
        label: "Elizabeth Tower".to_string(),
        lines,
        tint: LandmarkTint::Neutral,
    }
}

fn sydney_scene(twinkle: char, compact: bool) -> LandmarkScene {
    let lines = if compact {
        vec![" SYDNEY ".to_string(), format!("  ^{}^   ", twinkle)]
    } else {
        vec![
            "      _/\\_   _/\\_       ".to_string(),
            "   __/ /\\ \\_/ /\\ \\__    ".to_string(),
            "  /__/ /  \\___/  \\__\\   ".to_string(),
            "  \\  \\ \\  /   \\  /  /   ".to_string(),
            "   \\__\\_\\/_____/\\_/     ".to_string(),
            "      /_/       \\_\\     ".to_string(),
            format!("  ~~~~~{}~~~~~~~~~~~    ", twinkle),
        ]
    };

    LandmarkScene {
        label: "Sydney Opera House".to_string(),
        lines,
        tint: LandmarkTint::Cool,
    }
}

fn city_signature_scene(
    name_norm: &str,
    is_day: bool,
    phase: u64,
    compact: bool,
    width: usize,
    height: usize,
) -> LandmarkScene {
    let seed = hash64(name_norm);
    let tag = city_tag(name_norm);

    if compact {
        let mini = mini_skyline(seed, 8);
        return LandmarkScene {
            label: format!("{tag} Signature"),
            lines: vec![format!(" {tag} "), format!(" {mini} ")],
            tint: LandmarkTint::Neutral,
        };
    }

    let scene_width = width.clamp(16, 30);
    let skyline_rows = height.saturating_sub(2).clamp(4, 7);
    let sky = animated_sky(is_day, phase, scene_width);
    let city = build_skyline_rows(seed, phase, scene_width, skyline_rows);

    let mut lines = Vec::with_capacity(skyline_rows + 2);
    lines.push(sky);
    lines.extend(city);
    lines.push(format!(
        "{}{}",
        "~".repeat(scene_width.saturating_sub(8)),
        city_wave_char(seed, phase)
    ));

    LandmarkScene {
        label: format!("{tag} Signature"),
        lines,
        tint: LandmarkTint::Neutral,
    }
}

fn build_skyline_rows(seed: u64, phase: u64, width: usize, rows: usize) -> Vec<String> {
    let mut heights = vec![0usize; width];
    let mut x = 0usize;
    let mut rng = seed;
    let max_h = rows.max(1);

    while x < width {
        rng = lcg(rng);
        let building_w = 2 + (rng as usize % 4);
        rng = lcg(rng);
        let building_h = 1 + (rng as usize % max_h);
        let end = (x + building_w).min(width);
        for h in heights.iter_mut().take(end).skip(x) {
            *h = building_h;
        }
        x = end.saturating_add(1);
    }

    let mut out = Vec::with_capacity(rows);
    for row in (1..=rows).rev() {
        let mut line = String::with_capacity(width);
        for (col, h) in heights.iter().copied().enumerate() {
            if h < row {
                line.push(' ');
                continue;
            }

            let is_roof = h == row;
            if is_roof {
                line.push(if (seed + col as u64).is_multiple_of(5) {
                    '^'
                } else {
                    '_'
                });
            } else {
                let lit = (seed + phase + col as u64 + row as u64).is_multiple_of(7);
                line.push(if lit { 'o' } else { '|' });
            }
        }
        out.push(line);
    }
    out
}

fn animated_sky(is_day: bool, phase: u64, width: usize) -> String {
    let mut line = vec![' '; width];
    if is_day {
        let sun_x = (phase as usize % width.max(1)).min(width.saturating_sub(1));
        line[sun_x] = 'o';
        if width > 8 {
            let cloud = "~~";
            let c_start = (sun_x + 4).min(width.saturating_sub(cloud.len()));
            for (i, ch) in cloud.chars().enumerate() {
                line[c_start + i] = ch;
            }
        }
    } else {
        let star_count = (width / 6).max(2);
        for i in 0..star_count {
            let x = ((phase as usize + i * 5) % width.max(1)).min(width.saturating_sub(1));
            line[x] = if i.is_multiple_of(2) { '*' } else { '.' };
        }
    }
    line.into_iter().collect()
}

fn mini_skyline(seed: u64, width: usize) -> String {
    let mut out = String::with_capacity(width);
    let mut r = seed;
    for _ in 0..width {
        r = lcg(r);
        out.push(match r % 4 {
            0 => '_',
            1 => '|',
            2 => '^',
            _ => '.',
        });
    }
    out
}

fn city_wave_char(seed: u64, phase: u64) -> char {
    if (seed + phase).is_multiple_of(2) {
        '~'
    } else {
        '-'
    }
}

fn hash64(input: &str) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for b in input.as_bytes() {
        hash ^= u64::from(*b);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn lcg(seed: u64) -> u64 {
    seed.wrapping_mul(6364136223846793005).wrapping_add(1)
}

fn city_tag(name_norm: &str) -> String {
    let joined = name_norm
        .split_whitespace()
        .take(3)
        .map(|w| w.chars().take(3).collect::<String>())
        .collect::<Vec<_>>()
        .join(" ");
    if joined.is_empty() {
        "LOCAL".to_string()
    } else {
        joined.to_ascii_uppercase()
    }
}

fn normalize(input: &str) -> String {
    input
        .trim()
        .to_ascii_lowercase()
        .replace(['-', '_', ','], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn fit_line(line: &str, width: usize) -> String {
    let mut out = line.chars().take(width).collect::<String>();
    let len = out.chars().count();
    if len < width {
        out.push_str(&" ".repeat(width - len));
    }
    out
}
