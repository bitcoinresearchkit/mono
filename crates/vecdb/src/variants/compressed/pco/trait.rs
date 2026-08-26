use pco::data_types::Number;

/// Converts a vec value to and from its pco number representation.
///
/// # Safety
///
/// When `IS_TRANSPARENT` is true, `Self` and `NumberType` must have identical
/// layouts, and every valid `NumberType` bit pattern must also be a valid
/// `Self` bit pattern. PCO uses that guarantee to encode and decode directly
/// from the value buffer.
pub unsafe trait Pco: Sized {
    type NumberType: Number;

    const IS_TRANSPARENT: bool = false;

    fn to_number(self) -> Self::NumberType;

    fn from_number(value: Self::NumberType) -> crate::Result<Self>;
}
