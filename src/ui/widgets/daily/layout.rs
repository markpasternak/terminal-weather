use super::*;

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Copy)]
pub(super) struct DailyLayout {
    pub(super) show_icon: bool,
    pub(super) show_bar: bool,
    pub(super) show_header: bool,
    pub(super) show_precip_col: bool,
    pub(super) show_gust_col: bool,
    pub(super) bar_width: usize,
    pub(super) column_spacing: u16,
}

impl DailyLayout {
    pub(super) fn for_area(area: Rect) -> Self {
        let inner_width = area.width.saturating_sub(2) as usize;
        if inner_width >= 112 {
            return Self::wide(inner_width);
        } else if inner_width >= 86 {
            return Self::medium_plus(inner_width);
        } else if inner_width >= 56 {
            return Self::medium(inner_width);
        } else if inner_width >= 36 {
            return Self::compact(inner_width);
        }
        Self::narrow()
    }

    pub(super) fn max_rows(self, inner_height: u16) -> usize {
        let reserved = u16::from(self.show_header);
        usize::from(inner_height.saturating_sub(reserved)).min(7)
    }

    fn wide(inner_width: usize) -> Self {
        let bar_width = inner_width
            .saturating_sub(4 + 3 + 5 + 5 + 5 + 4 + 10)
            .clamp(18, 48);
        Self {
            show_icon: true,
            show_bar: true,
            show_header: true,
            show_precip_col: true,
            show_gust_col: true,
            bar_width,
            column_spacing: 2,
        }
    }

    fn medium_plus(inner_width: usize) -> Self {
        let bar_width = inner_width
            .saturating_sub(4 + 3 + 5 + 5 + 5 + 8)
            .clamp(14, 34);
        Self {
            show_icon: true,
            show_bar: true,
            show_header: true,
            show_precip_col: true,
            show_gust_col: false,
            bar_width,
            column_spacing: 1,
        }
    }

    fn medium(inner_width: usize) -> Self {
        let bar_width = inner_width.saturating_sub(4 + 3 + 5 + 5 + 6).clamp(10, 24);
        Self {
            show_icon: true,
            show_bar: true,
            show_header: true,
            show_precip_col: false,
            show_gust_col: false,
            bar_width,
            column_spacing: 1,
        }
    }

    fn compact(inner_width: usize) -> Self {
        let bar_width = inner_width.saturating_sub(4 + 5 + 5 + 3).clamp(6, 18);
        Self {
            show_icon: false,
            show_bar: true,
            show_header: true,
            show_precip_col: false,
            show_gust_col: false,
            bar_width,
            column_spacing: 1,
        }
    }

    fn narrow() -> Self {
        Self {
            show_icon: false,
            show_bar: false,
            show_header: false,
            show_precip_col: false,
            show_gust_col: false,
            bar_width: 0,
            column_spacing: 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn for_area_selects_expected_layout_breakpoints() {
        let wide = DailyLayout::for_area(area_with_inner_width(112));
        assert!(wide.show_icon);
        assert!(wide.show_bar);
        assert!(wide.show_header);
        assert!(wide.show_precip_col);
        assert!(wide.show_gust_col);
        assert_eq!(wide.column_spacing, 2);
        assert!((18..=48).contains(&wide.bar_width));

        let medium_plus = DailyLayout::for_area(area_with_inner_width(86));
        assert!(medium_plus.show_icon);
        assert!(medium_plus.show_bar);
        assert!(medium_plus.show_header);
        assert!(medium_plus.show_precip_col);
        assert!(!medium_plus.show_gust_col);
        assert_eq!(medium_plus.column_spacing, 1);
        assert!((14..=34).contains(&medium_plus.bar_width));

        let medium = DailyLayout::for_area(area_with_inner_width(56));
        assert!(medium.show_icon);
        assert!(medium.show_bar);
        assert!(medium.show_header);
        assert!(!medium.show_precip_col);
        assert!(!medium.show_gust_col);
        assert_eq!(medium.column_spacing, 1);
        assert!((10..=24).contains(&medium.bar_width));

        let compact = DailyLayout::for_area(area_with_inner_width(36));
        assert!(!compact.show_icon);
        assert!(compact.show_bar);
        assert!(compact.show_header);
        assert!(!compact.show_precip_col);
        assert!(!compact.show_gust_col);
        assert_eq!(compact.column_spacing, 1);
        assert!((6..=18).contains(&compact.bar_width));

        let narrow = DailyLayout::for_area(area_with_inner_width(35));
        assert!(!narrow.show_icon);
        assert!(!narrow.show_bar);
        assert!(!narrow.show_header);
        assert!(!narrow.show_precip_col);
        assert!(!narrow.show_gust_col);
        assert_eq!(narrow.bar_width, 0);
        assert_eq!(narrow.column_spacing, 1);
    }

    #[test]
    fn max_rows_respects_header_and_seven_day_cap() {
        let wide = DailyLayout::for_area(area_with_inner_width(112));
        assert_eq!(wide.max_rows(20), 7);
        assert_eq!(wide.max_rows(3), 2);

        let narrow = DailyLayout::for_area(area_with_inner_width(35));
        assert_eq!(narrow.max_rows(20), 7);
        assert_eq!(narrow.max_rows(3), 3);
    }

    fn area_with_inner_width(inner_width: u16) -> Rect {
        Rect::new(0, 0, inner_width + 2, 12)
    }
}
