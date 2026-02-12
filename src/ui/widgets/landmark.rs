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
        skyline_scene(is_day, phase, compact)
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
        vec!["  [STO]  ".to_string(), format!("  ~{}~~   ", twinkle)]
    } else {
        vec![
            "      |>>>      ".to_string(),
            "      |         ".to_string(),
            "   ___|___      ".to_string(),
            "  /_/_|_\\_\\     ".to_string(),
            "  |  _ _  |     ".to_string(),
            "  | | | | |___  ".to_string(),
            "  |_|_|_|_|___| ".to_string(),
            format!("  ~~~{}~~~~~~~ ", twinkle),
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
        vec!["  [PAR]  ".to_string(), format!("   {}     ", twinkle)]
    } else {
        vec![
            "      /\\        ".to_string(),
            "     /  \\       ".to_string(),
            "    / /\\ \\      ".to_string(),
            "   / /  \\ \\     ".to_string(),
            "  /_/____\\_\\    ".to_string(),
            "     |  |       ".to_string(),
            "   __|__|__     ".to_string(),
            format!("  ~~~~{}~~~~    ", twinkle),
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
            " [NYC] ".to_string(),
            format!(" {}{}{}{}{}  ", window, window, window, window, window),
        ]
    } else {
        vec![
            "      __|__      ".to_string(),
            "  []  |___|  []  ".to_string(),
            format!("  ||  |{}{}{}|  ||  ", window, window, window),
            " _||__|___|__||_ ".to_string(),
            " |  |  | |  |  | ".to_string(),
            " |[]|[]| |[]|[]| ".to_string(),
            " |__|__|_|__|__| ".to_string(),
            "~~~~~~~~~~~~~~~~ ".to_string(),
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
        vec![" [TKY] ".to_string(), format!("  /{}\\   ", twinkle)]
    } else {
        vec![
            "      /\\         ".to_string(),
            "     /  \\        ".to_string(),
            "    /====\\       ".to_string(),
            "      ||         ".to_string(),
            "    __||__       ".to_string(),
            "   |  ||  |      ".to_string(),
            "   |__||__|      ".to_string(),
            format!(" ~~~~~{}~~~~~    ", twinkle),
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
        vec![" [LDN] ".to_string(), format!("  [{}]   ", twinkle)]
    } else {
        vec![
            "      []         ".to_string(),
            "      ||         ".to_string(),
            "   ___||___      ".to_string(),
            "  |  __   |      ".to_string(),
            "  | |  |  |      ".to_string(),
            "  | |  |  |__    ".to_string(),
            "  |_|__|__|__|   ".to_string(),
            format!(" ~~~~{}~~~~~~~   ", twinkle),
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
        vec![
            " [SYD] ".to_string(),
            format!("  {}{}{}    ", '^', twinkle, '^'),
        ]
    } else {
        vec![
            "     _/\\_        ".to_string(),
            "   _/ /\\ \\_      ".to_string(),
            " _/ _/  \\_ \\_    ".to_string(),
            "/__/      \\__\\   ".to_string(),
            "   \\  /\\  /      ".to_string(),
            "    \\/  \\/       ".to_string(),
            format!(" ~~~~{}~~~~~~~   ", twinkle),
        ]
    };

    LandmarkScene {
        label: "Sydney Opera House".to_string(),
        lines,
        tint: LandmarkTint::Cool,
    }
}

fn skyline_scene(is_day: bool, phase: u64, compact: bool) -> LandmarkScene {
    let cloud_offset = (phase % 6) as usize;
    let sky = if is_day {
        let mut line = "   .--.          ".to_string();
        let cloud = " .-(    ). ";
        if cloud_offset + cloud.len() < line.len() {
            line.replace_range(cloud_offset..(cloud_offset + cloud.len()), cloud);
        }
        line
    } else {
        let star_pos = (phase % 10) as usize + 2;
        let mut line = "                ".to_string();
        if star_pos < line.len() {
            line.replace_range(star_pos..(star_pos + 1), "*");
        }
        line
    };

    let lines = if compact {
        vec![" [CITY] ".to_string(), " _|_|_  ".to_string()]
    } else {
        vec![
            sky,
            "   _   _    _    ".to_string(),
            "  | |_| |__| |   ".to_string(),
            "  |  _  / _` |   ".to_string(),
            "  | | | | (_| |  ".to_string(),
            "  |_| |_|\\__,_|  ".to_string(),
            " ~~~~~~~~~~~~~~~ ".to_string(),
        ]
    };

    LandmarkScene {
        label: "Local Skyline".to_string(),
        lines,
        tint: LandmarkTint::Neutral,
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
