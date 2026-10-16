# BRK PCO fork

This crate is copied from PCO `v1.0.3-1` at commit
`0491110a7356fa59e35e9f38e5bb631bf35fa05c`.

BRK's patch threads `MaybeUninit<T>` through decompression and exposes
`read_uninit`. This lets `vecdb` decode transparent values directly into a
destination `Vec`'s spare capacity without first initializing or copying the
output buffer. The existing initialized `read` API remains available and uses
the same decoder.

Keep unrelated upstream code unchanged so future upstream comparisons and
updates remain mechanical.

A second patch makes the encoder's page dissection sound
(`chunk_latent_compressor.rs`): the four per-page buffers (`ans_vals`,
`ans_bits`, `offsets`, `offset_bits`) are filled through their spare capacity
as `MaybeUninit` and get their length only once every element is written,
instead of `set_len` before initialization. The ANS buffers stay empty when the
latent variable needs no ANS (they were left uninitialized and never read).
Output bytes are unchanged.

Upstream fixes kept small and local:

- `mode/dict.rs`: dictionary indices are validated and joined over the batch
  only (`[..dst.len()]`), not the stale tail of the 256-entry scratch, which
  made valid short dictionary pages fail as corrupt.
- `delta/conv1.rs`: a page shorter than the conv1 order pads its delta state
  with zeros (as consecutive deltas do) instead of panicking, and a final
  encoding batch shorter than the order predicts nothing (the previous batch
  already did) instead of panicking.
- `macros.rs`: `define_number_enum!` is no longer exported: dtype_dispatch's
  reference-enum constructors do not tie the input lifetime to the enum's, so
  outside code could keep a reference past its storage. The match macros stay
  exported.
- `describers.rs`: float types describe `Dict` mode chunks (they panicked).
- `standalone/decompressor.rs`: `FileDecompressor::simple_decompress` is public
  so `with_max_prealloc` reaches a public entry point.
- `chunk_latent_decompressor.rs`: a latent variable with one bin and no offset
  bits (a constant page) requests no padded body reader, so its batches skip
  the 9 KiB zeroed end-of-input buffer (constant pages decode about 30%
  faster).
