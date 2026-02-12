use proptest::prelude::*;
use terminal_weather::ui::widgets::daily::bar_bounds;

proptest! {
    #[test]
    fn bar_bounds_never_overflow(
        min in -80.0f32..80.0,
        max in -80.0f32..80.0,
        global_min in -100.0f32..40.0,
        global_max in 41.0f32..120.0,
        width in 1usize..80usize,
    ) {
        let local_min = min.min(max);
        let local_max = min.max(max);
        let g_min = global_min.min(global_max - 0.1);
        let g_max = global_max.max(g_min + 0.1);

        let (start, end) = bar_bounds(local_min, local_max, g_min, g_max, width);
        prop_assert!(start <= width);
        prop_assert!(end <= width);
        prop_assert!(start <= end);
    }
}
