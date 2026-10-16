use bitview_primitives::with_op_return_kinds;
use bitview_traversable::Traversable;

/// One value per OP_RETURN kind: fields follow `OpReturnKind` order, so values indexed by kind
/// push in field order.
macro_rules! define_by_kind {
    ($($kind:ident => $field:ident, $description:literal,)+) => {
        #[derive(Clone, Traversable)]
        pub struct ByKind<T> {
            $(
                #[doc = $description]
                pub $field: T,
            )+
        }

        impl<T> ByKind<T> {
            pub fn try_new<E>(mut create: impl FnMut(&'static str) -> Result<T, E>) -> Result<Self, E> {
                Ok(Self { $($field: create(stringify!($field))?),+ })
            }

            pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
                [$(&mut self.$field),+].into_iter()
            }
        }
    };
}

with_op_return_kinds!(define_by_kind);
