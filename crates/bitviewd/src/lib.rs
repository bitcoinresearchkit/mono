#![doc = include_str!("../README.md")]

use brk_error::Result;

use bitview::{ComputePluginSet, ImportContext, QueryPluginSet};
use brk_reader::Reader;
use vecdb::{Exit, ReadOnlyClone};

mod config;
mod paths;

use crate::config::Config;

/// Runs the Bitview daemon process with the supplied composition.
pub fn run<P>(import: impl FnMut(ImportContext<'_>, &Reader) -> Result<P>) -> Result<()>
where
    P: ComputePluginSet + ReadOnlyClone,
    P::ReadOnly: QueryPluginSet + 'static,
{
    let config = Config::import()?;

    brk_logger::init(Some(&config.server.data_path.join("logs")))?;

    let exit = Exit::new();
    exit.set_ctrlc_handler();

    bitview::run(config, exit, import)
}
