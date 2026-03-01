with open('src/ui/widgets/daily/summary.rs', 'r') as f:
    content = f.read()

content = content.replace(
"""fn actionability_summary(summary: &WeekSummaryData) -> String {
    if summary.precip_total >= 20.0 {
        "Precip-heavy week: prioritize dry windows".to_string()
    } else if summary.breeziest_txt != "--" && summary.breeziest_txt.contains("m/s") {
        "Mixed week: track wind and UV day by day".to_string()
    } else {
        "Stable week: low planning friction".to_string()
    }
}""",
"""fn actionability_summary(summary: &WeekSummaryData) -> &'static str {
    if summary.precip_total >= 20.0 {
        "Precip-heavy week: prioritize dry windows"
    } else if summary.breeziest_txt != "--" && summary.breeziest_txt.contains("m/s") {
        "Mixed week: track wind and UV day by day"
    } else {
        "Stable week: low planning friction"
    }
}""")

with open('src/ui/widgets/daily/summary.rs', 'w') as f:
    f.write(content)
