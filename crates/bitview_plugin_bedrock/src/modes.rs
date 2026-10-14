use bitview_traversable::Traversable;

use super::ModeId;

#[derive(Traversable)]
pub struct Modes<T> {
    /// Bedrock's unweighted mode gives every unspent satoshi equal weight. It uses the
    /// all-chain distribution of UTXO creation prices and the unweighted share
    /// of supply whose creation price exceeds spot. A UTXO's creation price is
    /// Bitcoin's spot price when that output was created.
    pub unweighted: T,
    /// Bedrock's awake mode weights each UTXO age range by wakefulness—the
    /// share of its accumulated coin days that has been consumed—and
    /// calibrates against the resulting weighted share of supply in loss.
    pub awake: T,
    /// Bedrock's mobile mode weights each UTXO age range by mobility—the
    /// estimated probability that UTXOs of that age will ever be spent—and
    /// calibrates against the resulting weighted share of supply in loss.
    pub mobile: T,
}

impl<T> Modes<T> {
    pub fn from_fn(mut create: impl FnMut(ModeId) -> T) -> Self {
        Self {
            unweighted: create(ModeId::Unweighted),
            awake: create(ModeId::Awake),
            mobile: create(ModeId::Mobile),
        }
    }

    pub fn try_from_fn<E>(mut create: impl FnMut(ModeId) -> Result<T, E>) -> Result<Self, E> {
        Ok(Self {
            unweighted: create(ModeId::Unweighted)?,
            awake: create(ModeId::Awake)?,
            mobile: create(ModeId::Mobile)?,
        })
    }

    pub fn select_mut(&mut self, id: ModeId) -> &mut T {
        match id {
            ModeId::Unweighted => &mut self.unweighted,
            ModeId::Awake => &mut self.awake,
            ModeId::Mobile => &mut self.mobile,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        [&self.unweighted, &self.awake, &self.mobile].into_iter()
    }

    pub fn select(&self, id: ModeId) -> &T {
        match id {
            ModeId::Unweighted => &self.unweighted,
            ModeId::Awake => &self.awake,
            ModeId::Mobile => &self.mobile,
        }
    }
}
