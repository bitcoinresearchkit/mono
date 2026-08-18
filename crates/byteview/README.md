# brk_byteview

BRK's immutable byte slice. Its Rust library name remains `byteview`.

Values up to 20 bytes on 64-bit targets are stored directly in the 24-byte
view. Larger values and their subslices share one reference-counted allocation.
This keeps BRK's small database keys allocation-free while allowing decoded
table blocks to expose cheap owned subslices.

This is a specialized fork of
[`byteview`](https://github.com/fjall-rs/byteview), maintained for the
[Bitcoin Research Kit](https://bitcoinresearchkit.org).
