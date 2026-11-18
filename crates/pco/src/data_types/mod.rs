pub(crate) use dynamic::LatentType;
pub use dynamic::NumberType;
pub(crate) use split_latents::SplitLatents;

use crate::data_types::latent_priv::LatentPriv;
use crate::data_types::number_priv::NumberPriv;

mod dynamic;
pub(crate) mod float;
pub(crate) mod latent_priv;
pub(crate) mod number_priv;
pub(crate) mod signed;
mod split_latents;
pub(crate) mod unsigned;

/// Trait for types that behave like unsigned integers.
pub trait Latent: LatentPriv {}

impl<L: LatentPriv> Latent for L {}

/// Trait for types that Pco can compress and decompress.
pub trait Number: NumberPriv {}

impl<T: NumberPriv> Number for T {}
