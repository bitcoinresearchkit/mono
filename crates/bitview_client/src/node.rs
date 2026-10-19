//! The typed series tree's building blocks: leaf traits, lazily built nodes, and the `shape!` and
//! `accessor!` macros the generated tree is declared with.

use std::{
    ops::Deref,
    sync::{Arc, OnceLock},
};

use bitview_primitives::Index;

use crate::{BitviewClientBase, SeriesEndpoint};

/// Non-generic trait for series patterns (usable in collections).
pub trait AnySeriesPattern {
    fn name(&self) -> &str;

    fn indexes(&self) -> &'static [Index];
}

pub trait SeriesPattern<T>: AnySeriesPattern {
    /// Get an endpoint builder for a specific index, if supported.
    fn get(&self, index: Index) -> Option<SeriesEndpoint<T>>;
}

pub trait SeriesValue {
    /// The value where it can be missing (e.g. a period without blocks): `Option<Self>`, or the
    /// value itself when it is already optional.
    type Nullable;
}

impl<T> SeriesValue for Option<T> {
    type Nullable = Option<T>;
}

/// A series-tree node or leaf, built from its series name or base.
pub trait Node: Sized + Send + Sync + 'static {
    fn build(client: Arc<BitviewClientBase>, name: String) -> Self;
}

/// A series-tree child, built on first access from its parent's client and base and its own name
/// template: `*` stands for the base, and with an empty base the `_` joining it goes too.
pub struct LazyNode<T> {
    client: Arc<BitviewClientBase>,
    base: Arc<str>,
    template: &'static str,
    node: OnceLock<Box<T>>,
}

impl<T> LazyNode<T> {
    pub fn new(client: &Arc<BitviewClientBase>, base: &Arc<str>, template: &'static str) -> Self {
        Self {
            client: client.clone(),
            base: base.clone(),
            template,
            node: OnceLock::new(),
        }
    }

    fn name(&self) -> String {
        let (base, template) = (&*self.base, self.template);
        if !base.is_empty() {
            template.replacen('*', base, 1)
        } else if let Some(rest) = template.strip_prefix("*_") {
            rest.to_owned()
        } else {
            template.replacen("_*", "", 1).replacen('*', "", 1)
        }
    }
}

impl<T: Node> Deref for LazyNode<T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.node
            .get_or_init(|| Box::new(T::build(self.client.clone(), self.name())))
    }
}

impl<P: AnySeriesPattern + Node> AnySeriesPattern for LazyNode<P> {
    fn name(&self) -> &str {
        (**self).name()
    }

    fn indexes(&self) -> &'static [Index] {
        (**self).indexes()
    }
}

impl<T, P: SeriesPattern<T> + Node> SeriesPattern<T> for LazyNode<P> {
    fn get(&self, index: Index) -> Option<SeriesEndpoint<T>> {
        (**self).get(index)
    }
}

/// A series-tree shape: a struct with one lazy field per child, documented with its canonical path.
macro_rules! shape {
    ($name:ident $(<$($p:ident),+>)? at $at:literal { $($field:ident: $ty:ty = $template:literal,)* }) => {
        #[doc = concat!("Series-tree node, e.g. at `", $at, "`.")]
        pub struct $name $(<$($p),+>)? { $(pub $field: $crate::LazyNode<$ty>,)* }
        impl $(<$($p: Send + Sync + 'static),+>)? $crate::Node for $name $(<$($p),+>)? where $($ty: $crate::Node,)* {
            fn build(client: ::std::sync::Arc<$crate::BitviewClientBase>, base: String) -> Self {
                let base: ::std::sync::Arc<str> = base.into();
                Self { $($field: $crate::LazyNode::new(&client, &base, $template),)* }
            }
        }
    };
}

/// A leaf accessor: the series name plus one endpoint method per index (`date` for date indexes),
/// each with its value type (`T::Nullable` where values can be missing); `get` takes the weakest.
macro_rules! accessor {
    ($name:ident, $by:ident, $indexes:ident, $any:ty { $($kind:ident $method:ident: $index:ident -> $value:ty,)* }) => {
        pub struct $by<T> {
            client: ::std::sync::Arc<$crate::BitviewClientBase>,
            name: ::std::sync::Arc<str>,
            _marker: ::std::marker::PhantomData<T>,
        }
        impl<T: ::serde::de::DeserializeOwned + $crate::SeriesValue<Nullable: ::serde::de::DeserializeOwned>> $by<T> {
            $(accessor!(@method $kind $method $index $value);)*
        }
        pub struct $name<T> {
            pub by: $by<T>,
        }
        impl<T> $name<T> {
            pub fn new(client: ::std::sync::Arc<$crate::BitviewClientBase>, name: String) -> Self {
                Self { by: $by { client, name: name.into(), _marker: ::std::marker::PhantomData } }
            }
            pub fn name(&self) -> &str {
                &self.by.name
            }
        }
        impl<T> $crate::AnySeriesPattern for $name<T> {
            fn name(&self) -> &str {
                &self.by.name
            }
            fn indexes(&self) -> &'static [$crate::Index] {
                $indexes
            }
        }
        impl<T: ::serde::de::DeserializeOwned + $crate::SeriesValue<Nullable: ::serde::de::DeserializeOwned>> $crate::SeriesPattern<$any> for $name<T> {
            fn get(&self, index: $crate::Index) -> Option<$crate::SeriesEndpoint<$any>> {
                $indexes.contains(&index).then(|| $crate::SeriesEndpoint::new(self.by.client.clone(), self.by.name.clone(), index))
            }
        }
        impl<T: Send + Sync + 'static> $crate::Node for $name<T> {
            fn build(client: ::std::sync::Arc<$crate::BitviewClientBase>, name: String) -> Self {
                Self::new(client, name)
            }
        }
    };
    (@method date $method:ident $index:ident $value:ty) => {
        pub fn $method(&self) -> $crate::DateSeriesEndpoint<$value> {
            $crate::SeriesEndpoint::new(self.client.clone(), self.name.clone(), $crate::Index::$index)
        }
    };
    (@method plain $method:ident $index:ident $value:ty) => {
        pub fn $method(&self) -> $crate::SeriesEndpoint<$value> {
            $crate::SeriesEndpoint::new(self.client.clone(), self.name.clone(), $crate::Index::$index)
        }
    };
}
