#[cfg(target_pointer_width = "64")]
use std::mem;

use std::{
    io::{Cursor, ErrorKind, Read, Result},
    thread,
};

use super::{ByteView, HeapAllocationHeader, INLINE_SIZE, LongRepr, ShortRepr, Trailer};

#[test]
#[expect(
    clippy::indexing_slicing,
    reason = "bounds follow the generated input length"
)]
fn inline_boundary_and_shared_subslices() {
    for len in [0, 1, 8, 10, INLINE_SIZE, INLINE_SIZE + 1, 20, 21, 64] {
        let bytes: Vec<_> = (0..=u8::MAX).take(len).collect();
        let view = ByteView::new(&bytes);
        assert_eq!(view.is_inline(), len <= INLINE_SIZE);
        assert_eq!(&*view, bytes);
        for split in 0..=len {
            let fused = ByteView::fused(&bytes[..split], &bytes[split..]);
            assert_eq!(view, fused);
        }
    }

    let bytes: Vec<_> = (0..128).collect();
    let parent = ByteView::new(&bytes);
    let child = parent.slice(13..100).slice(7..70);
    drop(parent);
    thread::scope(|scope| {
        for _ in 0..4 {
            let copy = child.clone();
            scope.spawn(move || {
                for _ in 0..100 {
                    assert_eq!(&*copy.clone().slice(3..40), &(23..60).collect::<Vec<_>>());
                }
            });
        }
    });
    assert_eq!(&*child, &bytes[20..83]);
    assert_eq!(child.ref_count(), 1);
}

#[test]
fn heap_views_share_and_short_slices_copy_inline() {
    let view = ByteView::from("helloworld_thisisalongstring");
    let clone = view.clone();
    let tail = view.slice(11..);
    assert_eq!(&*tail, b"thisisalongstring");
    assert_eq!(view.ref_count(), 3);

    // A slice that fits inline copies its bytes instead of holding the allocation.
    let short = tail.slice(..=4);
    assert!(short.is_inline());
    assert_eq!(&*short, b"thisi");
    assert_eq!(view.ref_count(), 3);

    drop(clone);
    drop(view);
    assert_eq!(tail.ref_count(), 1);
    drop(tail);
    assert_eq!(&*short, b"thisi");

    let inline = ByteView::from("abcdef");
    let copy = inline.clone();
    assert_eq!(&*inline.slice(3..), b"def");
    assert_eq!(&*inline.slice(0..0), b"");
    drop(inline);
    assert_eq!((&*copy, copy.ref_count()), (&b"abcdef"[..], 1));
    assert_eq!(&*ByteView::default(), b"");
}

#[test]
fn ordering_matches_byte_slices() {
    let samples = [
        &[][..],
        &[0],
        &[0, 0],
        b"abc",
        b"abcdef",
        b"def",
        &[255, 255, 12, 255],
        &[255, 255, 12, 255, 0],
        b"abcdefabcdefabcdefabcde",
        b"abcdefabcdefabcdefabcdef",
        b"abcdefabcdefabcdefabcdeg",
        b"abcdefhelloworldhelloworld",
        b"xycdefabcdefabcdefabcdef",
        &[255; 12],
        &[255; 13],
        &[
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0,
        ],
    ];
    for left in samples {
        for right in samples {
            let (view_left, view_right) = (ByteView::from(left), ByteView::from(right));
            assert_eq!(view_left.cmp(&view_right), left.cmp(right));
            assert_eq!(view_left == view_right, left == right);
        }
    }
}

#[test]
#[should_panic(expected = "initializer returned a different buffer")]
fn initializer_rejects_another_buffer() {
    let _ = ByteView::try_init(64, |_| Ok::<_, ()>(&[0; 64]));
}

#[test]
#[should_panic(expected = "initializer returned a partial buffer")]
fn initializer_rejects_partial_success() {
    let _ = ByteView::try_init(64, |_| Ok::<_, ()>(&[]));
}

struct InspectingReader;

impl Read for InspectingReader {
    fn read(&mut self, buffer: &mut [u8]) -> Result<usize> {
        assert!(buffer.iter().all(|byte| *byte == 0));
        buffer.fill(42);
        Ok(buffer.len())
    }
}

#[test]
fn builders_and_readers_only_expose_initialized_bytes() -> Result<()> {
    for len in [0, INLINE_SIZE, INLINE_SIZE + 1, 4096] {
        let mut builder = ByteView::builder(len);
        assert!(builder.iter().all(|byte| *byte == 0));
        builder.fill(42);
        let view = builder.freeze();
        assert_eq!(view, ByteView::from_reader(&mut InspectingReader, len)?);
        assert_eq!(&*view.clone(), vec![42; len]);
    }
    assert!(matches!(
        ByteView::from_reader(&mut Cursor::new([1, 2, 3]), 4096),
        Err(error) if error.kind() == ErrorKind::UnexpectedEof
    ));
    // A failing initializer may leave a partial write behind; it must still free cleanly.
    let failed = ByteView::try_init(64, |buffer| {
        if let Some(byte) = buffer.first_mut() {
            byte.write(1);
        }
        Err(())
    });
    assert!(failed.is_err());
    Ok(())
}

#[test]
#[cfg(target_pointer_width = "64")]
fn memsize() {
    assert_eq!(mem::size_of::<ShortRepr>(), mem::size_of::<LongRepr>());
    assert_eq!(mem::size_of::<Trailer>(), mem::size_of::<LongRepr>());

    assert_eq!(16, mem::size_of::<ByteView>());
    assert_eq!(
        32,
        mem::size_of::<ByteView>() + mem::size_of::<HeapAllocationHeader>()
    );
}
