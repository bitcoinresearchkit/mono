//! Best-quality brotli copies of the files served, each made once, in the background. Nothing
//! else the server sends is compressed, a CDN in front does that; a file doesn't change, so
//! brotli's best makes a page as big as Studio's (2.6 MB) about a fifth of its size, made once
//! instead of at every response. Until a copy is made, the file goes out as it is. A file has one
//! copy, of its current content (an edit replaces it), and at most two are made at once.

use std::{
    collections::HashMap,
    io::Cursor,
    path::{Path, PathBuf},
    sync::{Mutex, PoisonError},
    thread,
};

use axum::{body::Bytes, http::HeaderMap};
use brotli::enc::{BrotliCompress, BrotliEncoderParams};
use rapidhash::v3::rapidhash_v3;

/// Smaller files gain nothing worth a copy.
const MIN_LEN: usize = 1024;
/// Copies made at once: a deploy's first visits ask for every file.
const MAKING_AT_ONCE: usize = 2;

#[derive(Default)]
struct Copies {
    /// Each file's content hash and its copy, none while it's being made.
    by_file: HashMap<PathBuf, (u64, Option<Bytes>)>,
    making: usize,
}

static COPIES: Mutex<Option<Copies>> = Mutex::new(None);

/// The best brotli copy of `file`'s `contents` when the request takes brotli and it's made;
/// starts making it otherwise, when fewer than [`MAKING_AT_ONCE`] are being made.
pub(crate) fn copy(
    file: &Path,
    contents: &Bytes,
    content_type: &str,
    request: &HeaderMap,
) -> Option<Bytes> {
    if contents.len() < MIN_LEN || !compressible(content_type) || !accepts_brotli(request) {
        return None;
    }
    let id = rapidhash_v3(contents);
    let mut guard = COPIES.lock().unwrap_or_else(PoisonError::into_inner);
    let copies = guard.get_or_insert_with(Copies::default);
    if let Some((current, copy)) = copies.by_file.get(file)
        && *current == id
    {
        return copy.clone();
    }
    if copies.making == MAKING_AT_ONCE {
        return None;
    }
    copies.making += 1;
    copies.by_file.insert(file.to_owned(), (id, None));
    let (file, contents) = (file.to_owned(), contents.clone());
    thread::spawn(move || {
        let made = compress(&contents);
        let mut guard = COPIES.lock().unwrap_or_else(PoisonError::into_inner);
        let copies = guard.get_or_insert_with(Copies::default);
        copies.making -= 1;
        // The file may have changed meanwhile: its newer content's copy is the one to keep.
        if copies.by_file.get(&file).is_some_and(|(current, _)| *current == id) {
            match made {
                Some(bytes) => copies.by_file.insert(file, (id, Some(bytes))),
                None => copies.by_file.remove(&file),
            };
        }
    });
    None
}
fn compress(contents: &[u8]) -> Option<Bytes> {
    let params = BrotliEncoderParams {
        quality: 11,
        lgwin: 24,
        ..Default::default()
    };
    let mut out = Vec::with_capacity(contents.len() / 3);
    BrotliCompress(&mut Cursor::new(contents), &mut out, &params).ok()?;
    Some(out.into())
}

fn compressible(content_type: &str) -> bool {
    content_type.starts_with("text/")
        || matches!(
            content_type,
            "application/javascript"
                | "application/json"
                | "application/xml"
                | "application/manifest+json"
                | "image/svg+xml"
        )
}

/// Whether `br` is acceptable: named with a nonzero weight, or `*` without `br;q=0`.
fn accepts_brotli(request: &HeaderMap) -> bool {
    bitview_http::accepts_encoding(request, "br") == Some(true)
}
