const KIB: f64 = 1024.0;
const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];

/// The binary unit that keeps `max` between 1 and 1024: (divisor, suffix).
pub fn byte_unit(max: f64) -> (f64, &'static str) {
    let mut divisor = 1.0;
    for unit in UNITS {
        if max < divisor * KIB || unit == "TiB" {
            return (divisor, unit);
        }
        divisor *= KIB;
    }
    unreachable!()
}

pub fn bytes(value: f64) -> String {
    let (divisor, unit) = byte_unit(value);
    format!("{:.1} {unit}", value / divisor)
}

pub fn duration(seconds: f64) -> String {
    let seconds = seconds.max(0.0);
    if seconds < 10.0 {
        return format!("{seconds:.1}s");
    }
    if seconds < 59.5 {
        return format!("{seconds:.0}s");
    }
    let minutes = (seconds / 60.0).round() as u64;
    match (minutes / 60, minutes % 60) {
        (0, m) => format!("{m}m"),
        (h, m) => format!("{h}h{m:02}"),
    }
}

/// UTC date and time of a Unix timestamp, `YYYY-MM-DD HH:MM`.
pub fn datetime(unix: u64) -> String {
    let minutes = unix % 86_400 / 60;
    format!("{} {:02}:{:02}", date(unix), minutes / 60, minutes % 60)
}

/// Civil date of a Unix timestamp, `YYYY-MM-DD`.
fn date(unix: u64) -> String {
    let days = (unix / 86_400) as i64 + 719_468;
    let era = days.div_euclid(146_097);
    let day_of_era = days.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_index = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_index + 2) / 5 + 1;
    let month = if month_index < 10 {
        month_index + 3
    } else {
        month_index - 9
    };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    format!("{year:04}-{month:02}-{day:02}")
}

pub fn count(value: f64) -> String {
    match value.abs() {
        v if v >= 1e6 => format!("{:.1}M", value / 1e6),
        v if v >= 1e3 => format!("{:.0}k", value / 1e3),
        v if v >= 10.0 || v == 0.0 => format!("{value:.0}"),
        _ => format!("{value:.1}"),
    }
}
