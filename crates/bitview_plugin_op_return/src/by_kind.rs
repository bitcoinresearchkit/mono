use bitview_traversable::Traversable;
use brk_types::OpReturnKind;

macro_rules! define_by_kind {
    ($($field:ident => $kind:ident),+ $(,)?) => {
        #[derive(Clone, Traversable)]
        pub struct ByKind<T> {
            $(pub $field: T),+
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
    runes => Runes,
    veri_block => VeriBlock,
    omni => Omni,
    stacks => Stacks,
    blockstack => Blockstack,
    colu => Colu,
    open_assets => OpenAssets,
    komodo => Komodo,
    coin_spark => CoinSpark,
    poet => Poet,
    docproof => Docproof,
    open_timestamps => OpenTimestamps,
    factom => Factom,
    eternity_wall => EternityWall,
    memo => Memo,
    bitproof => Bitproof,
    ascribe => Ascribe,
    stampery => Stampery,
    epobc => Epobc,
    bare_hash => BareHash,
    text => Text,
    empty => Empty,
    unknown => Unknown,
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
