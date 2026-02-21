pub(in super::super) fn draw_ambient_cloud(
    canvas: &mut [Vec<char>],
    cx: usize,
    cy: usize,
    cloud_w: usize,
    rows: usize,
    canvas_w: usize,
    horizon_y: usize,
) {
    // Multi-row cloud with density falloff from center
    // Top row: lighter/thinner  ░░▒▒▒░░
    // Middle row: denser         ▒▓▓▓▓▓▒
    // Bottom row: wispy          ░░▒░░
    let patterns = cloud_patterns(rows);

    let cloud = CloudGeometry {
        cx,
        cy,
        width: cloud_w,
        canvas_width: canvas_w,
        horizon_y,
    };
    for (row_idx, pattern) in patterns.iter().enumerate() {
        if !paint_cloud_row(canvas, pattern, row_idx, &cloud) {
            break;
        }
    }
}

struct CloudGeometry {
    cx: usize,
    cy: usize,
    width: usize,
    canvas_width: usize,
    horizon_y: usize,
}

fn cloud_patterns(rows: usize) -> &'static [&'static [char]] {
    if rows >= 3 {
        &[
            &[' ', '░', '░', '▒', '▒', '░', '░', ' '],
            &['░', '▒', '▓', '▓', '▓', '▓', '▒', '░'],
            &[' ', ' ', '░', '▒', '▒', '░', ' ', ' '],
        ]
    } else {
        &[
            &[' ', '░', '▒', '▒', '▒', '░', ' '],
            &['░', '▒', '▓', '▓', '▒', '░', ' '],
        ]
    }
}

fn paint_cloud_row(
    canvas: &mut [Vec<char>],
    pattern: &[char],
    row_idx: usize,
    cloud: &CloudGeometry,
) -> bool {
    let y = cloud.cy + row_idx;
    if y >= cloud.horizon_y || y >= canvas.len() {
        return false;
    }
    let pat_len = pattern.len();
    for col in 0..cloud.width {
        let x = cloud.cx.wrapping_sub(cloud.width / 2).wrapping_add(col);
        if x >= cloud.canvas_width {
            continue;
        }
        let pat_idx = (col * pat_len) / cloud.width.max(1);
        let ch = pattern[pat_idx.min(pat_len - 1)];
        if ch != ' ' && canvas[y][x] == ' ' {
            canvas[y][x] = ch;
        }
    }
    true
}
