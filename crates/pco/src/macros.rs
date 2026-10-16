#![allow(clippy::single_component_path_imports)]

use dtype_dispatch::build_dtype_macros;

// BRK: not exported. dtype_dispatch's reference constructors (`new(&Container<S>)` on an
// `Enum<'a>`) do not tie the input lifetime to `'a`, so the definer stays crate-private.
build_dtype_macros!(
  define_number_enum,

  #[doc = "\
    Matches enums holding a container of `Number`s and puts the concrete type
    into scope.
  "]
  #[doc = "\
    You'll only want to use this if you're using pco's low level APIs.
    See the dtype_dispatch crate for usage details.
  "]
  #[macro_export]
  match_number_enum,

  Number,
  {
    F16 => half::f16,
    F32 => f32,
    F64 => f64,
    I8 => i8,
    I16 => i16,
    I32 => i32,
    I64 => i64,
    U8 => u8,
    U16 => u16,
    U32 => u32,
    U64 => u64,
  },
);

build_dtype_macros!(
  define_latent_enum,

  #[doc = "\
    Matches enums holding a container of `Latent`s and puts the concrete type
    into scope.
  "]
  #[doc = "\
    You'll only want to use this if you're using pco's low level APIs.
    See the dtype_dispatch crate for usage details.
  "]
  #[macro_export]
  match_latent_enum,

  Latent,
  {
    U8 => u8,
    U16 => u16,
    U32 => u32,
    U64 => u64,
  },
);

pub(crate) use {define_latent_enum, define_number_enum, match_latent_enum, match_number_enum};
