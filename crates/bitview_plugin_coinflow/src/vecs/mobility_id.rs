use super::Mobility;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MobilityId {
    Mobile,
    Immobile,
}

impl MobilityId {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Mobile => "mobile",
            Self::Immobile => "immobile",
        }
    }

    pub fn try_from_fn<T, E>(
        mut create: impl FnMut(Self) -> Result<T, E>,
    ) -> Result<Mobility<T>, E> {
        Ok(Mobility {
            mobile: create(Self::Mobile)?,
            immobile: create(Self::Immobile)?,
        })
    }
}
