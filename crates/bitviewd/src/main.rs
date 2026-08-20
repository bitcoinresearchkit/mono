#![doc = include_str!("../README.md")]

use bitview_default::DefaultPlugins;
use brk_error::Result;

fn main() -> Result<()> {
    bitviewd::run(DefaultPlugins::import)
}
