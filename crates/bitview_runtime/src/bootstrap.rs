use std::{collections::HashSet, fs, path::Path};

use bitview_plugin::{ImportContext, PluginId, PluginStorage, UpdateContext};
use brk_alloc::Mimalloc;
use brk_error::{Error, Result};
use tracing::info;

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
        info!("Removing unused plugin data: {path:?}");
        if entry.file_type()?.is_dir() {
            fs::remove_dir_all(path)?;
        } else {
            fs::remove_file(path)?;
        }
    }
    Ok(())
}

/// Imports and fully computes a composition before it can be published.
///
/// A composition may request one or more complete drop/reimport cycles to
/// reclaim transient memory accumulated during its initial computation.
/// Every entry under the shared plugin-data directory that is not claimed by
/// the imported composition is removed before computation begins.
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
        let mut plugins = import(import_context)?;
        let mut plugin_ids = Vec::new();
        plugins.for_each_plugin(&mut |plugin| plugin_ids.push(plugin.id()));
        sync_plugin_dirs(&plugins_path, plugin_ids.into_iter())?;

        match bootstrap_update(&mut plugins, update_context)? {
            BootstrapAction::Ready if needs_final_reimport => {
                needs_final_reimport = false;
            }
            BootstrapAction::Ready => return Ok(plugins),
            BootstrapAction::Reimport => {
                needs_final_reimport = true;
            }
        }

        info!("Dropping and reimporting plugins to reclaim memory...");
        drop(plugins);
        Mimalloc::collect();
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    use bitview_plugin::PluginId;
    use vecdb::Exit;

    use super::*;

    #[derive(crate::PluginSet)]
    struct TestPlugins {
        #[plugin_set(skip)]
        import: usize,
        #[plugin_set(skip)]
        computes: Arc<AtomicUsize>,
        #[plugin_set(skip)]
        reimport_first: bool,
    }

    impl ComputePluginSet for TestPlugins {
        fn bootstrap_compute(&mut self, _context: UpdateContext<'_>) -> Result<BootstrapAction> {
            self.computes.fetch_add(1, Ordering::Relaxed);
            Ok(if self.reimport_first && self.import == 1 {
                BootstrapAction::Reimport
            } else {
                BootstrapAction::Ready
            })
        }

        fn compute(&mut self, _context: UpdateContext<'_>) -> Result<()> {
            unreachable!()
        }
    }

    #[test]
    fn reimport_is_followed_by_a_clean_ready_import() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let imports = Arc::new(AtomicUsize::new(0));
        let computes = Arc::new(AtomicUsize::new(0));
        let import_context = ImportContext::new(directory.path());
        let exit = Exit::new();
        let plugins = bootstrap(
            import_context,
            |_context| {
                Ok(TestPlugins {
                    import: imports.fetch_add(1, Ordering::Relaxed) + 1,
                    computes: computes.clone(),
                    reimport_first: true,
                })
            },
            UpdateContext::new(&exit),
        )?;

        assert_eq!(plugins.import, 3);
        assert_eq!(imports.load(Ordering::Relaxed), 3);
        assert_eq!(computes.load(Ordering::Relaxed), 3);
        Ok(())
    }

    #[test]
    fn ready_composition_is_returned_without_reimport() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let imports = Arc::new(AtomicUsize::new(0));
        let computes = Arc::new(AtomicUsize::new(0));
        let import_context = ImportContext::new(directory.path());
        let exit = Exit::new();
        let plugins = bootstrap(
            import_context,
            |_context| {
                Ok(TestPlugins {
                    import: imports.fetch_add(1, Ordering::Relaxed) + 1,
                    computes: computes.clone(),
                    reimport_first: false,
                })
            },
            UpdateContext::new(&exit),
        )?;

        assert_eq!(plugins.import, 1);
        assert_eq!(imports.load(Ordering::Relaxed), 1);
        assert_eq!(computes.load(Ordering::Relaxed), 1);
        Ok(())
    }

    #[test]
    fn cleanup_retains_only_claimed_plugin_data() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let context = ImportContext::new(directory.path());
        let plugins = PluginStorage::plugins_path(context);
        let blocks = plugins.join("blocks");
        let stale = plugins.join("stale");
        fs::create_dir_all(&blocks)?;
        fs::create_dir(&stale)?;
        fs::write(blocks.join("marker"), b"current")?;
        fs::write(stale.join("marker"), b"stale")?;
        fs::write(plugins.join("orphan"), b"stale")?;

        sync_plugin_dirs(
            &plugins,
            [PluginId::new("blocks"), PluginId::new("constants")].into_iter(),
        )?;

        assert_eq!(fs::read(blocks.join("marker"))?, b"current");
        assert!(plugins.join("constants").is_dir());
        assert!(!stale.exists());
        assert!(!plugins.join("orphan").exists());
        Ok(())
    }

    #[test]
    fn duplicate_plugin_ids_are_rejected_before_sync() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let context = ImportContext::new(directory.path());
        let plugins = PluginStorage::plugins_path(context);
        let blocks = PluginId::new("blocks");

        assert!(sync_plugin_dirs(&plugins, [blocks, blocks].into_iter()).is_err());
        assert!(!plugins.exists());
        Ok(())
    }
}
