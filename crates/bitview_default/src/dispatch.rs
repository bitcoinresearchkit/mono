use crate::serial;

/// `rayon::join`, except in the serial CPU replay (see [`serial`]): there the branches run one
/// after the other, so a step's process CPU is its own. The parallel work inside each plugin is
/// unchanged.
pub fn join<A: Send, B: Send>(
    left: impl FnOnce() -> A + Send,
    right: impl FnOnce() -> B + Send,
) -> (A, B) {
    if serial::enabled() {
        (left(), right())
    } else {
        rayon::join(left, right)
    }
}
