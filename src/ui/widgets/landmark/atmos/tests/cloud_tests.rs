use super::super::paint_cloud_layer;
use super::test_support::blank_canvas;

#[test]
fn paint_cloud_layer_low_cloud_pct_no_op() {
    let mut canvas = blank_canvas(40, 16);
    // cloud_pct < 5 → early return
    paint_cloud_layer(&mut canvas, 3.0, 10.0, 0, 10, 40);
}

#[test]
fn paint_cloud_layer_small_horizon_no_op() {
    let mut canvas = blank_canvas(40, 16);
    // horizon_y < 4 → early return
    paint_cloud_layer(&mut canvas, 50.0, 10.0, 0, 3, 40);
}

#[test]
fn paint_cloud_layer_high_coverage_covers_more_of_canvas() {
    let mut light_canvas = blank_canvas(60, 20);
    let mut heavy_canvas = blank_canvas(60, 20);
    // cloud_pct > 80 → w/2 max_cloud_w
    paint_cloud_layer(&mut light_canvas, 30.0, 5.0, 0, 12, 60);
    paint_cloud_layer(&mut heavy_canvas, 90.0, 5.0, 0, 12, 60);
    // heavy coverage should produce at least as many cloud chars
    let light_cloud_chars = light_canvas
        .iter()
        .flatten()
        .filter(|c| !matches!(**c, ' '))
        .count();
    let heavy_cloud_chars = heavy_canvas
        .iter()
        .flatten()
        .filter(|c| !matches!(**c, ' '))
        .count();
    assert!(
        heavy_cloud_chars >= light_cloud_chars,
        "heavy clouds should fill more space"
    );
}

#[test]
fn paint_cloud_layer_medium_coverage_exercises_50_branch() {
    let mut canvas = blank_canvas(60, 20);
    // cloud_pct > 50 but <= 80 → w/3 max_cloud_w; cloud_pct <= 70 → 2 rows
    paint_cloud_layer(&mut canvas, 65.0, 8.0, 10, 14, 60);
}

#[test]
fn paint_cloud_layer_very_high_coverage_exercises_70_branch() {
    let mut canvas = blank_canvas(60, 20);
    // cloud_pct > 70 → 3 cloud_rows
    paint_cloud_layer(&mut canvas, 75.0, 8.0, 10, 14, 60);
}
