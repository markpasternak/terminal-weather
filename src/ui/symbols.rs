use crate::cli::IconMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemanticSymbol {
    Fresh,
    Stale,
    Offline,
    TrendUp,
    TrendDown,
    SeverityInfo,
    SeverityWarning,
    SeverityDanger,
    ConfidenceHigh,
    ConfidenceMedium,
    ConfidenceLow,
    Wind,
}

#[must_use]
pub fn symbol(symbol: SemanticSymbol, mode: IconMode) -> &'static str {
    match mode {
        IconMode::Unicode => unicode_symbol(symbol),
        IconMode::Ascii => ascii_symbol(symbol),
        IconMode::Emoji => emoji_symbol(symbol),
        IconMode::NerdFont => nerd_font_symbol(symbol),
    }
}

const fn unicode_symbol(symbol: SemanticSymbol) -> &'static str {
    match symbol {
        SemanticSymbol::Fresh | SemanticSymbol::ConfidenceHigh => "●",
        SemanticSymbol::Stale | SemanticSymbol::ConfidenceMedium => "◐",
        SemanticSymbol::Offline | SemanticSymbol::ConfidenceLow => "○",
        _ => unicode_non_status_symbol(symbol),
    }
}

const fn unicode_non_status_symbol(symbol: SemanticSymbol) -> &'static str {
    match symbol {
        SemanticSymbol::TrendUp => "↗",
        SemanticSymbol::TrendDown => "↘",
        SemanticSymbol::SeverityInfo => "ℹ",
        SemanticSymbol::SeverityWarning => "⚠",
        SemanticSymbol::SeverityDanger => "⛔",
        SemanticSymbol::Wind => "➤",
        SemanticSymbol::Fresh
        | SemanticSymbol::Stale
        | SemanticSymbol::Offline
        | SemanticSymbol::ConfidenceHigh
        | SemanticSymbol::ConfidenceMedium
        | SemanticSymbol::ConfidenceLow => unreachable!(),
    }
}

const fn ascii_symbol(symbol: SemanticSymbol) -> &'static str {
    match symbol {
        SemanticSymbol::Fresh => "OK",
        SemanticSymbol::Stale => "ST",
        SemanticSymbol::Offline => "OF",
        SemanticSymbol::TrendUp => "^",
        SemanticSymbol::TrendDown => "v",
        SemanticSymbol::Wind => ">",
        SemanticSymbol::SeverityInfo
        | SemanticSymbol::SeverityWarning
        | SemanticSymbol::SeverityDanger
        | SemanticSymbol::ConfidenceHigh
        | SemanticSymbol::ConfidenceMedium
        | SemanticSymbol::ConfidenceLow => ascii_indicator_symbol(symbol),
    }
}

const fn ascii_indicator_symbol(symbol: SemanticSymbol) -> &'static str {
    match symbol {
        SemanticSymbol::SeverityInfo => "i",
        SemanticSymbol::SeverityWarning => "!",
        SemanticSymbol::SeverityDanger => "X",
        SemanticSymbol::ConfidenceHigh => "H",
        SemanticSymbol::ConfidenceMedium => "M",
        SemanticSymbol::ConfidenceLow => "L",
        _ => unreachable!(),
    }
}

const fn emoji_symbol(symbol: SemanticSymbol) -> &'static str {
    match symbol {
        SemanticSymbol::Fresh | SemanticSymbol::ConfidenceHigh => "🟢",
        SemanticSymbol::Stale | SemanticSymbol::ConfidenceMedium => "🟡",
        SemanticSymbol::Offline => "🔴",
        SemanticSymbol::ConfidenceLow => "⚪",
        _ => emoji_non_status_symbol(symbol),
    }
}

const fn emoji_non_status_symbol(symbol: SemanticSymbol) -> &'static str {
    match symbol {
        SemanticSymbol::TrendUp => "📈",
        SemanticSymbol::TrendDown => "📉",
        SemanticSymbol::SeverityInfo => "ℹ️",
        SemanticSymbol::SeverityWarning => "⚠️",
        SemanticSymbol::SeverityDanger => "🛑",
        SemanticSymbol::Wind => "🧭",
        SemanticSymbol::Fresh
        | SemanticSymbol::Stale
        | SemanticSymbol::Offline
        | SemanticSymbol::ConfidenceHigh
        | SemanticSymbol::ConfidenceMedium
        | SemanticSymbol::ConfidenceLow => unreachable!(),
    }
}

fn nerd_font_symbol(symbol: SemanticSymbol) -> &'static str {
    use nerd_font_symbols::weather::WEATHER_STRONG_WIND;
    match symbol {
        SemanticSymbol::Fresh | SemanticSymbol::ConfidenceHigh => "●",
        SemanticSymbol::Stale | SemanticSymbol::ConfidenceMedium => "◐",
        SemanticSymbol::Offline | SemanticSymbol::ConfidenceLow => "○",
        _ => nerd_font_non_status_symbol(symbol, WEATHER_STRONG_WIND),
    }
}

fn nerd_font_non_status_symbol(symbol: SemanticSymbol, wind: &'static str) -> &'static str {
    match symbol {
        SemanticSymbol::TrendUp => "↑",
        SemanticSymbol::TrendDown => "↓",
        SemanticSymbol::SeverityInfo => "ℹ",
        SemanticSymbol::SeverityWarning => "⚠",
        SemanticSymbol::SeverityDanger => "⛔",
        SemanticSymbol::Wind => wind,
        SemanticSymbol::Fresh
        | SemanticSymbol::Stale
        | SemanticSymbol::Offline
        | SemanticSymbol::ConfidenceHigh
        | SemanticSymbol::ConfidenceMedium
        | SemanticSymbol::ConfidenceLow => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::{SemanticSymbol, symbol};
    use crate::cli::IconMode;

    #[test]
    fn symbols_have_ascii_parity() {
        let symbols = [
            SemanticSymbol::Fresh,
            SemanticSymbol::Stale,
            SemanticSymbol::Offline,
            SemanticSymbol::TrendUp,
            SemanticSymbol::TrendDown,
            SemanticSymbol::SeverityInfo,
            SemanticSymbol::SeverityWarning,
            SemanticSymbol::SeverityDanger,
            SemanticSymbol::ConfidenceHigh,
            SemanticSymbol::ConfidenceMedium,
            SemanticSymbol::ConfidenceLow,
            SemanticSymbol::Wind,
        ];

        for value in symbols {
            assert!(!symbol(value, IconMode::Unicode).is_empty());
            assert!(!symbol(value, IconMode::Ascii).is_empty());
            assert!(!symbol(value, IconMode::Emoji).is_empty());
            assert!(!symbol(value, IconMode::NerdFont).is_empty());
        }
    }
}
