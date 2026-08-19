mod base;

pub use base::*;

fn period_suffix(period: &str) -> String {
    if period.is_empty() {
        String::new()
    } else {
        format!("_{period}")
    }
}
