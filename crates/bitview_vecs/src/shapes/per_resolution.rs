use bitview_primitives::with_resolution_fields;
use bitview_traversable::Traversable;

macro_rules! define_per_resolution {
    (
        periods { $($field:ident: $index:ident => $param:ident,)* }
        epochs { $($epoch:ident: $epoch_index:ident => $epoch_param:ident,)* }
    ) => {
        #[derive(Clone, Traversable)]
        #[traversable(merge)]
        pub struct PerResolution<$($param,)* $($epoch_param,)*> {
            $(pub $field: $param,)*
            $(pub $epoch: $epoch_param,)*
        }
    };
}

with_resolution_fields!(define_per_resolution);
