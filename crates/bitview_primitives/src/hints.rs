#[inline(always)]
pub(crate) fn unlikely(value: bool) -> bool {
    if value {
        cold();
    }
    value
}

#[cold]
fn cold() {}
