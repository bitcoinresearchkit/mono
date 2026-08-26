use std::mem::MaybeUninit;
use std::ops::Range;

use crate::data_types::{Latent, Number};
use crate::macros::{define_latent_enum, define_number_enum, match_number_enum};

type Arr<T> = [T];
type UninitArr<T> = [MaybeUninit<T>];

define_latent_enum!(
  #[derive()]
  pub DynLatentSlice(&Arr)
);

define_number_enum!(
  #[derive()]
  pub DynNumberSlice(&Arr)
);

impl<'a> DynNumberSlice<'a> {
  pub fn len(&self) -> usize {
    match_number_enum!(
      self,
      DynNumberSlice<T>(inner) => {
        inner.len()
      }
    )
  }

  pub fn slice(&self, range: Range<usize>) -> Self {
    match_number_enum!(
      self,
      DynNumberSlice<T>(inner) => {
        DynNumberSlice::new(&inner[range])
      }
    )
  }
}

define_number_enum!(
  #[derive()]
  pub DynNumberSliceMut(&mut UninitArr)
);

impl<'a> DynNumberSliceMut<'a> {
  pub fn from_initialized<T: Number>(values: &'a mut [T]) -> Self {
    // SAFETY: MaybeUninit<T> has the same layout as T, and may always contain
    // an initialized T value.
    let values = unsafe {
      std::slice::from_raw_parts_mut(
        values.as_mut_ptr().cast::<MaybeUninit<T>>(),
        values.len(),
      )
    };
    Self::new(values)
  }

  pub fn len(&self) -> usize {
    match_number_enum!(
      self,
      DynNumberSliceMut<T>(inner) => {
        inner.len()
      }
    )
  }
}
