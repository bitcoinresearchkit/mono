use bitview_traversable::Traversable;

/// A monetary value in its integer cents and dollar representations.
/// The series is the USD value; cents are its exact storage.
#[derive(Clone, Traversable)]
#[traversable(merge)]
pub struct Fiat<C, U> {
    /// Reported in US dollars.
    pub usd: U,
    /// Reported in US cents; 100 cents equal one US dollar.
    #[traversable(hidden)]
    pub cents: C,
}
