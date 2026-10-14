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
