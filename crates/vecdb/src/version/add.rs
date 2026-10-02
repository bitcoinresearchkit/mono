use std::ops::Add;

use super::Version;

impl Add<Version> for Version {
    type Output = Self;
    fn add(self, rhs: Version) -> Self::Output {
        Self(self.0.wrapping_add(rhs.0))
    }
}
