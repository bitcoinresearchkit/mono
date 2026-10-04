use std::{collections::HashSet, fs, path::Path};

use bitview_plugin::{ImportContext, PluginId, PluginStorage, UpdateContext};
use brk_error::{Error, Result};
use tracing::{info, warn};

use crate::{BootstrapAction, ComputePluginSet, update::bootstrap_update};

fn sync_plugin_dirs(plugins_path: &Path, plugin_ids: impl Iterator<Item = PluginId>) -> Result<()> {
    let mut active_ids = HashSet::new();
    for id in plugin_ids {
        if !active_ids.insert(id) {
            return Err(Error::Internal("Duplicate plugin ID"));
        }
    }

    fs::create_dir_all(plugins_path)?;
    for id in &active_ids {
        fs::create_dir_all(plugins_path.join(id.as_str()))?;
    }

    for entry in fs::read_dir(plugins_path)? {
        let entry = entry?;
        let name = entry.file_name();
        if active_ids.iter().any(|id| name == id.as_str()) {
            continue;
        }

        let path = entry.path();
        if entry.file_type()?.is_dir() {
            fs::remove_dir_all(&path)?;
        } else {
            fs::remove_file(&path)?;
        }
        warn!("Removed unused plugin data at {}", path.display());
    }
    Ok(())
}

/// Imports and fully computes a composition before it can be published.
///
/// A composition may request one or more complete drop/reimport cycles to
/// reclaim transient memory accumulated during its initial computation.
/// Every entry under the shared plugin-data directory that is not claimed by
/// the imported composition is removed before computation begins.
///
/// Each import runs under the shutdown lock, since imports write (resets, compaction, data
/// removal): a soft quit waits for it, so an import must not block on outside services.
pub fn bootstrap<P>(
    import_context: ImportContext<'_>,
    mut import: impl FnMut(ImportContext<'_>) -> Result<P>,
    update_context: UpdateContext<'_>,
) -> Result<P>
where
    P: ComputePluginSet,
{
    let plugins_path = PluginStorage::plugins_path(import_context);
    let mut needs_final_reimport = false;

    loop {
        let lock = import_context.exit().lock();
        let mut plugins = import(import_context)?;
        let mut plugin_ids = Vec::new();
        plugins.for_each_plugin(&mut |plugin| plugin_ids.push(plugin.id()));
        sync_plugin_dirs(&plugins_path, plugin_ids.into_iter())?;
        drop(lock);

        match bootstrap_update(&mut plugins, update_context)? {
            BootstrapAction::Ready if needs_final_reimport => {
                needs_final_reimport = false;
            }
            BootstrapAction::Ready => return Ok(plugins),
            BootstrapAction::Reimport => {
                needs_final_reimport = true;
            }
        }

        info!("Reloading plugins to release temporary startup memory...");
        drop(plugins);
        brk_alloc::collect();
    }
}
