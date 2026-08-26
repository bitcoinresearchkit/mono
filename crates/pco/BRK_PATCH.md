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
