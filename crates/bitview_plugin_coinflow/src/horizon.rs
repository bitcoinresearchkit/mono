use bitview_traversable::Traversable;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HorizonId {
    Y8,
    Y4,
    Y2,
    Y1,
    M6,
    M3,
    M1,
}

impl HorizonId {
    pub const ALL: [Self; 7] = [
        Self::Y8,
        Self::Y4,
        Self::Y2,
        Self::Y1,
        Self::M6,
        Self::M3,
        Self::M1,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Self::Y8 => "8y",
            Self::Y4 => "4y",
            Self::Y2 => "2y",
            Self::Y1 => "1y",
            Self::M6 => "6m",
            Self::M3 => "3m",
            Self::M1 => "1m",
        }
    }

    pub const fn days(self) -> f64 {
        match self {
            Self::Y8 => 8.0 * 365.0,
            Self::Y4 => 4.0 * 365.0,
            Self::Y2 => 2.0 * 365.0,
            Self::Y1 => 365.0,
            Self::M6 => 180.0,
            Self::M3 => 90.0,
            Self::M1 => 30.0,
        }
    }

    pub fn select<T>(self, horizons: &Horizons<T>) -> &T {
        match self {
            Self::Y8 => &horizons._8y,
            Self::Y4 => &horizons._4y,
            Self::Y2 => &horizons._2y,
            Self::Y1 => &horizons._1y,
            Self::M6 => &horizons._6m,
            Self::M3 => &horizons._3m,
            Self::M1 => &horizons._1m,
        }
    }

    pub fn select_mut<T>(self, horizons: &mut Horizons<T>) -> &mut T {
        match self {
            Self::Y8 => &mut horizons._8y,
            Self::Y4 => &mut horizons._4y,
            Self::Y2 => &mut horizons._2y,
            Self::Y1 => &mut horizons._1y,
            Self::M6 => &mut horizons._6m,
            Self::M3 => &mut horizons._3m,
            Self::M1 => &mut horizons._1m,
        }
    }

    pub fn from_fn<T>(mut create: impl FnMut(Self) -> T) -> Horizons<T> {
        Horizons {
            _8y: create(Self::Y8),
            _4y: create(Self::Y4),
            _2y: create(Self::Y2),
            _1y: create(Self::Y1),
            _6m: create(Self::M6),
            _3m: create(Self::M3),
            _1m: create(Self::M1),
        }
    }

    pub fn try_from_fn<T, E>(
        mut create: impl FnMut(Self) -> Result<T, E>,
    ) -> Result<Horizons<T>, E> {
        Ok(Horizons {
            _8y: create(Self::Y8)?,
            _4y: create(Self::Y4)?,
            _2y: create(Self::Y2)?,
            _1y: create(Self::Y1)?,
            _6m: create(Self::M6)?,
            _3m: create(Self::M3)?,
            _1m: create(Self::M1)?,
        })
    }
}

#[derive(Clone, Copy, Traversable)]
pub struct Horizons<T> {
    /// Uses an eight-year forward spending horizon.
    pub _8y: T,
    /// Uses a four-year forward spending horizon.
    pub _4y: T,
    /// Uses a two-year forward spending horizon.
    pub _2y: T,
    /// Uses a one-year forward spending horizon.
    pub _1y: T,
    /// Uses a 180-day forward spending horizon.
    pub _6m: T,
    /// Uses a 90-day forward spending horizon.
    pub _3m: T,
    /// Uses a 30-day forward spending horizon.
    pub _1m: T,
}
