/// Imports multiple items in parallel using thread::scope.
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
        std::thread::scope(|s| -> brk_error::Result<_> {
            $(let $name = s.spawn(|| $expr);)+
            Ok(($($name.join().unwrap()?,)+))
        })?
    }};
}
