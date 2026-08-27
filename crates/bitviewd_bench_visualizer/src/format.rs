const KIB: f64 = 1024.0;
const MIB: f64 = KIB * 1024.0;
const GIB: f64 = MIB * 1024.0;

const MINUTE: f64 = 60.0;
const HOUR: f64 = 3600.0;

/// Returns (scaled_value, unit_suffix)
pub fn bytes(bytes: f64) -> (f64, &'static str) {
    if bytes >= GIB {
        (bytes / GIB, "GiB")
    } else if bytes >= MIB {
        (bytes / MIB, "MiB")
    } else if bytes >= KIB {
        (bytes / KIB, "KiB")
    } else {
        (bytes, "bytes")
    }
}

/// Returns (scaled_value, divisor, axis_label)
pub fn time(seconds: f64) -> (f64, f64, &'static str) {
    if seconds >= HOUR * 2.0 {
        (seconds / HOUR, HOUR, "Time (h)")
    } else if seconds >= MINUTE * 2.0 {
        (seconds / MINUTE, MINUTE, "Time (min)")
    } else {
        (seconds, 1.0, "Time (s)")
    }
}

pub fn axis_number(value: f64) -> String {
    if value >= 1000.0 {
        let k = value / 1000.0;
        if k.fract() == 0.0 || k >= 100.0 {
            format!("{:.0}k", k)
        } else if k >= 10.0 {
            format!("{:.1}k", k)
        } else {
            format!("{:.2}k", k)
        }
    } else {
        format!("{:.0}", value)
    }
}
