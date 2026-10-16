# Unsafe usage

Every `unsafe` block carries `#[expect(unsafe_code, reason = ...)]`; this lists
them with the invariant each relies on.

- `table/util.rs` `compare_prefixed_slice`: unchecked sub-slices of `prefix`
  and `needle`, bounded by `max_pfx_len = min(prefix.len(), needle.len())`.
- `table/filter/bit_array/builder.rs` `enable_bit`: unchecked byte write; the
  Bloom builder reduces every bit index modulo its own bit count.
- `double_ended_peekable.rs`: `unreachable_unchecked` right after the
  `Unpeeked` state is replaced with `Peeked`.

Block decoding has no unchecked access. Blocks are not checksummed, so on-disk
bytes are untrusted: `Block::from_file`/`from_reader` validate the trailer,
binary index layout and restart pointers of every item block once, on load
(`Trailer::validate`, an `InvalidTrailer` error), and record extents decoded
from varints are bounds-checked when sliced, so a corrupt record panics
instead of reading out of bounds.

## Checks

```bash
cargo clippy -p brk_lsm_tree --all-targets --all-features -- -D warnings
```
