#[inline]
pub fn likely(b: bool) -> bool {
    if !b {
        cold();
    }
    b
}

#[inline]
pub fn unlikely(b: bool) -> bool {
    if b {
        cold();
    }
    b
}

#[cold]
fn cold() {}
