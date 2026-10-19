pub use decoding::Decoder;
pub use encoding::{quantize_weights, Encoder};
pub use spec::Spec;

mod decoding;
mod encoding;
mod spec;

// must be u16 or larger
// should not be exposed in public API
pub(crate) type AnsState = u32;
// must be u16 or larger
// should not be exposed in public API
pub(crate) type Symbol = u32;
