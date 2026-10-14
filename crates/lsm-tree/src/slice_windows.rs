// Copyright (c) 2024-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

pub trait GrowingWindowsExt<T> {
    fn growing_windows<'a>(&'a self) -> impl Iterator<Item = &'a [T]>
    where
        T: 'a;
}

impl<T> GrowingWindowsExt<T> for [T] {
    fn growing_windows<'a>(&'a self) -> impl Iterator<Item = &'a [T]>
    where
        T: 'a,
    {
        (1..=self.len()).flat_map(|size| self.windows(size))
    }
}

pub trait ShrinkingWindowsExt<T> {
    fn shrinking_windows<'a>(&'a self) -> impl Iterator<Item = &'a [T]>
    where
        T: 'a;
}

impl<T> ShrinkingWindowsExt<T> for [T] {
    fn shrinking_windows<'a>(&'a self) -> impl Iterator<Item = &'a [T]>
    where
        T: 'a,
    {
        (1..=self.len()).rev().flat_map(|size| self.windows(size))
    }
}
