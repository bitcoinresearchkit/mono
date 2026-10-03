//! Global allocator and memory utilities for brk.
//!
//! Binaries choose mimalloc themselves:
//! `#[global_allocator] static GLOBAL: brk_alloc::MiMalloc = brk_alloc::MiMalloc;`.
//! This crate never installs an allocator, so libraries depending on it leave the choice to the
//! final binary. [`collect`] only affects memory mimalloc manages.

use libmimalloc_sys::mi_collect;

pub use mimalloc::MiMalloc;

/// Eagerly returns mimalloc's free memory to the OS.
/// Only call at natural pause points.
#[inline]
pub fn collect() {
    unsafe { mi_collect(true) }
}
