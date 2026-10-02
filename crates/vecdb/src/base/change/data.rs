use crate::Stamp;

/// Parsed change data returned by parse_change_data, consumed by apply_rollback.
#[derive(Debug)]
pub struct ChangeData<T> {
    pub prev_stamp: Stamp,
    pub prev_stored_len: usize,
    pub truncated_start: usize,
    pub truncated_values: Vec<T>,
    pub prev_pushed: Vec<T>,
}

impl<T> ChangeData<T> {
    /// Re-queues truncated values where disk stops agreeing with the rolled-back
    /// state. Query persisted length only when there are truncated values.
    pub fn into_rollback(self, real_stored_len: impl FnOnce() -> usize) -> (Stamp, usize, Vec<T>) {
        let (stored_len, pushed) = if self.truncated_values.is_empty() {
            (self.prev_stored_len, self.prev_pushed)
        } else {
            let agree_at = self.truncated_start.min(real_stored_len());
            let mut pushed = self.truncated_values;
            pushed.extend(self.prev_pushed);
            (agree_at, pushed)
        };
        (self.prev_stamp, stored_len, pushed)
    }
}
