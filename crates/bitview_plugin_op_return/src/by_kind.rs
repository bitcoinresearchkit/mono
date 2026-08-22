use bitview_traversable::Traversable;
use brk_types::OpReturnKind;

macro_rules! define_by_kind {
    ($($field:ident => $kind:ident, $description:literal),+ $(,)?) => {
        #[derive(Clone, Traversable)]
        pub struct ByKind<T> {
            $(
                #[doc = $description]
                pub $field: T
            ),+
        }

        impl<T> ByKind<T> {
            pub fn new(mut create: impl FnMut(OpReturnKind, &'static str) -> T) -> Self {
                Self {
                    $($field: create(OpReturnKind::$kind, stringify!($field))),+
                }
            }

            pub fn iter(&self) -> impl Iterator<Item = &T> {
                [$( &self.$field ),+].into_iter()
            }

            pub fn iter_typed(&self) -> impl Iterator<Item = (OpReturnKind, &T)> {
                [$( (OpReturnKind::$kind, &self.$field) ),+].into_iter()
            }
        }
    };
}

define_by_kind! {
    runes => Runes, "Restricted to OP_RETURN outputs classified as Runes by their payload opcode.",
    veri_block => VeriBlock, "Restricted to OP_RETURN outputs classified as VeriBlock by their 82-byte post-OP_RETURN payload.",
    omni => Omni, "Restricted to OP_RETURN outputs whose first pushed payload starts with the Omni marker.",
    stacks => Stacks, "Restricted to OP_RETURN outputs whose first pushed payload starts with a Stacks marker.",
    blockstack => Blockstack, "Restricted to OP_RETURN outputs whose first pushed payload starts with the Blockstack marker.",
    colu => Colu, "Restricted to OP_RETURN outputs whose first pushed payload starts with the Colu marker.",
    open_assets => OpenAssets, "Restricted to OP_RETURN outputs whose first pushed payload starts with the Open Assets marker.",
    komodo => Komodo, "Restricted to OP_RETURN outputs classified as Komodo by their 36-to-38-byte post-OP_RETURN payload.",
    coin_spark => CoinSpark, "Restricted to OP_RETURN outputs whose first pushed payload starts with the CoinSpark marker.",
    poet => Poet, "Restricted to OP_RETURN outputs whose first pushed payload starts with the Proof of Existence marker.",
    docproof => Docproof, "Restricted to OP_RETURN outputs whose first pushed payload starts with the Docproof marker.",
    open_timestamps => OpenTimestamps, "Restricted to OP_RETURN outputs whose first pushed payload starts with the OpenTimestamps marker.",
    factom => Factom, "Restricted to OP_RETURN outputs whose first pushed payload starts with the Factom marker.",
    eternity_wall => EternityWall, "Restricted to OP_RETURN outputs whose first pushed payload starts with the Eternity Wall marker.",
    memo => Memo, "Restricted to OP_RETURN outputs whose first pushed payload matches a recognized Memo action marker.",
    bitproof => Bitproof, "Restricted to OP_RETURN outputs whose first pushed payload starts with the Bitproof marker.",
    ascribe => Ascribe, "Restricted to OP_RETURN outputs whose first pushed payload starts with the Ascribe marker.",
    stampery => Stampery, "Restricted to OP_RETURN outputs whose first pushed payload starts with the Stampery marker.",
    epobc => Epobc, "Restricted to OP_RETURN outputs whose first pushed payload starts with the EPOBC marker.",
    bare_hash => BareHash, "Restricted to OP_RETURN outputs classified as bare hashes because their first pushed payload is 20 or 32 bytes.",
    text => Text, "Restricted to otherwise-unclassified OP_RETURN outputs whose first pushed payload is at least 90% printable ASCII.",
    empty => Empty, "Restricted to OP_RETURN outputs with no non-empty pushed payload.",
    unknown => Unknown, "Restricted to OP_RETURN outputs that do not match another recognized payload kind.",
}

#[cfg(test)]
mod tests {
    use brk_types::OpReturnKind;

    use super::ByKind;

    #[test]
    fn covers_every_kind_in_discriminant_order() {
        let by_kind = ByKind::new(|kind, _| kind);
        let kinds: Vec<_> = by_kind.iter_typed().collect();

        assert_eq!(kinds.len(), OpReturnKind::Unknown as usize + 1);
        for (index, (kind, value)) in kinds.into_iter().enumerate() {
            assert_eq!(kind as usize, index);
            assert_eq!(kind, *value);
        }
    }
}
