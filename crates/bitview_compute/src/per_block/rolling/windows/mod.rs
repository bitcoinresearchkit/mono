mod cached_window_start;
mod columnar;
mod columnar_from_1w;
mod rolling;
mod starts;
mod window_start;

pub use cached_window_start::CachedWindowStartVec;
pub use columnar::ColumnarRollingWindows;
pub use columnar_from_1w::ColumnarRollingWindowsFrom1w;
pub use rolling::RollingWindows;
pub use starts::WindowStarts;
pub use window_start::LazyWindowStartVec;

pub trait Lookback {
    fn start_vec(&self, days: usize) -> &LazyWindowStartVec;
}
