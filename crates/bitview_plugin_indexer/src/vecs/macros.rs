/// Imports multiple items in parallel on the rayon pool (nested imports share its threads).
/// Each expression must return `Result<T>`.
///
/// # Example
/// ```ignore
/// let (a, b, c) = parallel_import! {
///     a = SomeVec::import(&db, version),
///     b = OtherVec::import(&db, version),
///     c = ThirdVec::import(&db, version),
/// };
/// ```
macro_rules! parallel_import {
    ($($name:ident = $expr:expr),+ $(,)?) => {{
        $(let mut $name = None;)+
        rayon::scope(|s| {
            $(s.spawn(|_| $name = Some($expr));)+
        });
        ($($name.unwrap()?,)+)
    }};
}
