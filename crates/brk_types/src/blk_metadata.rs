use crate::BlkPosition;

#[derive(Debug)]
pub struct BlkMetadata {
    position: BlkPosition,
}

impl BlkMetadata {
    pub fn new(position: BlkPosition) -> Self {
        Self { position }
    }
    pub fn position(&self) -> BlkPosition {
        self.position
    }
}
