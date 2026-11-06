use std::ops::Add;

use super::Version;

/// The one composition: `combine`, an ordered mix (`a + b` isn't `b + a`) where no bump of one
/// component can cancel another's, as plain addition let (`(0 ⊕ 1) + 1` met `(0 ⊕ 2) + 2`).
impl Add<Version> for Version {
    type Output = Self;
    fn add(self, rhs: Version) -> Self::Output {
        self.combine(rhs)
    }
}
