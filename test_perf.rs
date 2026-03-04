fn parse_date_old(s: &str) -> Option<chrono::NaiveDate> {
    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
}

fn parse_date_new(s: &str) -> Option<chrono::NaiveDate> {
    if s.len() != 10 { return None; }
    let y = s[0..4].parse::<i32>().ok()?;
    let m = s[5..7].parse::<u32>().ok()?;
    let d = s[8..10].parse::<u32>().ok()?;
    chrono::NaiveDate::from_ymd_opt(y, m, d)
}

fn main() {
    let s = "2023-10-25";
    let t0 = std::time::Instant::now();
    for _ in 0..100000 {
        std::hint::black_box(parse_date_old(s));
    }
    println!("old: {:?}", t0.elapsed());

    let t0 = std::time::Instant::now();
    for _ in 0..100000 {
        std::hint::black_box(parse_date_new(s));
    }
    println!("new: {:?}", t0.elapsed());
}
