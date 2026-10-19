use crate::Stamp;

#[derive(Debug)]
pub struct ChangeData<T> {
    pub prev_stamp: Stamp,
    /// Rows below this one are unchanged since the previous stamp.
    pub truncated_start: usize,
    pub truncated_values: Vec<T>,
    pub prev_pushed: Vec<T>,
}
