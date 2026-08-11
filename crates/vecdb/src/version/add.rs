use std::ops::Add;

use super::Version;

impl Add<Version> for Version {
    type Output = Self;
    fn add(self, rhs: Version) -> Self::Output {
        Self(self.0.wrapping_add(rhs.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn addition_has_identical_debug_and_release_semantics() {
        assert_eq!(Version::new(u32::MAX) + Version::ONE, Version::ZERO);
    }
}
