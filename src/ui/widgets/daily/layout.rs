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
