# bitview_default

The default typed plugin graph and compute schedule used by Bitview.

The official [`bitviewd`](https://crates.io/crates/bitviewd) daemon selects this
set by default. Custom applications can reuse or extend `DefaultPlugins` and
run the resulting composition through [`bitview`](https://crates.io/crates/bitview).
