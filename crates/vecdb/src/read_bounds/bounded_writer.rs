use crate::{Result, ValueWriter};

use super::ReadBounds;

/// A value writer that cannot outlive its explicit read bounds.
pub struct BoundedWriter<'a> {
    writer: Box<dyn ValueWriter + 'a>,
    bounds: &'a ReadBounds,
}

impl<'a> BoundedWriter<'a> {
    pub(super) fn new(writer: Box<dyn ValueWriter + 'a>, bounds: &'a ReadBounds) -> Self {
        Self { writer, bounds }
    }
}

impl ValueWriter for BoundedWriter<'_> {
    fn write_next(&mut self, buf: &mut Vec<u8>) -> Result<()> {
        self.bounds.scope(|| self.writer.write_next(buf))
    }
}
