pub const MODE_COUNT: usize = 3;

/// A Bedrock mode: the unweighted distribution, or one weighted by wakefulness (Cointime) or
/// mobility (Coinflow).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum ModeId {
    Unweighted,
    Awake,
    Mobile,
}

impl ModeId {
    pub const ALL: [Self; MODE_COUNT] = [Self::Unweighted, Self::Awake, Self::Mobile];

    pub const fn name(self) -> &'static str {
        match self {
            Self::Unweighted => "unweighted",
            Self::Awake => "awake",
            Self::Mobile => "mobile",
        }
    }
}
