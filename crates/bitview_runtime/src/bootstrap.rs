use std::{collections::HashSet, fs, path::Path};

use bitview_plugin::{PLUGIN_DATA_DIR, PluginId};
use brk_alloc::Mimalloc;
use brk_error::{Error, Result};
use tracing::info;
use vecdb::Exit;

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
pub fn bootstrap<P>(
    outputs_path: &Path,
    mut import: impl FnMut() -> Result<P>,
    exit: &Exit,
) -> Result<P>
where
    P: ComputePluginSet,
{
    let plugins_path = outputs_path.join(PLUGIN_DATA_DIR);

    loop {
        let mut plugins = import()?;
        let mut plugin_ids = Vec::new();
        plugins.for_each_plugin(&mut |plugin| plugin_ids.push(plugin.id()));
        sync_plugin_dirs(&plugins_path, plugin_ids.into_iter())?;

        match bootstrap_update(&mut plugins, exit)? {
            BootstrapAction::Ready => return Ok(plugins),
            BootstrapAction::Reimport => {
                info!("Dropping and reimporting plugins to reclaim memory...");
                drop(plugins);
                Mimalloc::collect();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    use bitview_plugin::{Plugin, PluginId};

    use super::*;

    struct TestPlugins {
        import: usize,
        computes: Arc<AtomicUsize>,
    }

    impl crate::PluginSet for TestPlugins {
        fn for_each_plugin<'a>(&'a self, _visit: &mut dyn FnMut(&'a dyn Plugin)) {}
    }

    impl ComputePluginSet for TestPlugins {
        fn bootstrap_compute(&mut self, _exit: &Exit) -> Result<BootstrapAction> {
            self.computes.fetch_add(1, Ordering::Relaxed);
            Ok(if self.import == 1 {
                BootstrapAction::Reimport
            } else {
                BootstrapAction::Ready
            })
        }

        fn compute(&mut self, _exit: &Exit) -> Result<()> {
            unreachable!()
        }
    }

    #[test]
    fn reimports_before_returning_the_ready_composition() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let imports = Arc::new(AtomicUsize::new(0));
        let computes = Arc::new(AtomicUsize::new(0));
        let plugins = bootstrap(
            directory.path(),
            || {
                Ok(TestPlugins {
                    import: imports.fetch_add(1, Ordering::Relaxed) + 1,
                    computes: computes.clone(),
                })
            },
            &Exit::new(),
        )?;

        assert_eq!(plugins.import, 2);
        assert_eq!(imports.load(Ordering::Relaxed), 2);
        assert_eq!(computes.load(Ordering::Relaxed), 2);
        Ok(())
    }

    #[test]
    fn cleanup_retains_only_claimed_plugin_data() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let plugins = directory.path().join(PLUGIN_DATA_DIR);
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
        let plugins = directory.path().join(PLUGIN_DATA_DIR);
        let blocks = PluginId::new("blocks");

        assert!(sync_plugin_dirs(&plugins, [blocks, blocks].into_iter()).is_err());
        assert!(!plugins.exists());
        Ok(())
    }
}
