use std::{path::Path, time::Duration};

use bitview::update;
use bitview_custom_plugin_example::near_full_blocks::{Dependencies, ID, Vecs as NearFullBlocks};
use bitview_plugin::{ComputePlugin, ImportContext, Plugin, UpdateContext};
use bitview_query::Vecs as QueryVecs;
use bitview_runtime::{ComputePluginSet, PluginSet};
use bitview_traversable::Traversable;
use brk_error::{Error, Result};
use brk_types::{Height, Index, Version, Weight};
use vecdb::{AnyStoredVec, Database, Exit, ImportableVec, PAGE_SIZE, PcoVec, WritableVec};

#[derive(PluginSet, Traversable)]
struct TestPlugins {
    #[traversable(skip)]
    #[plugin_set(skip)]
    safe_height: Height,
    #[traversable(skip)]
    #[plugin_set(skip)]
    _source_db: Database,
    #[traversable(skip)]
    #[plugin_set(skip)]
    block_weights: PcoVec<Height, Weight>,
    near_full_blocks: NearFullBlocks,
    #[traversable(skip)]
    #[plugin_set(skip)]
    computed_while_closed: bool,
}

impl TestPlugins {
    fn import(outputs_path: &Path) -> Result<Self> {
        let source_db = Database::open(&outputs_path.join("source"))?;
        source_db.set_min_len(PAGE_SIZE)?;
        let block_weights = PcoVec::forced_import(&source_db, "block_weight", Version::ONE)?;

        Ok(Self {
            safe_height: Height::ZERO,
            _source_db: source_db,
            block_weights,
            near_full_blocks: NearFullBlocks::import(ImportContext::new(outputs_path))?,
            computed_while_closed: false,
        })
    }

    fn replace_tail(&mut self, safe_height: Height, tail: &[u32]) -> Result<()> {
        self.safe_height = safe_height;
        self.block_weights.truncate_if_needed(safe_height)?;
        for &weight in tail {
            self.block_weights.push(Weight::from(weight));
        }
        self.block_weights.write()?;
        Ok(())
    }

    fn queried_streak(&self) -> Result<Vec<u8>> {
        let query = QueryVecs::build(self);
        let entry = query
            .entry(&"near_full_block_streak".into(), Index::Height)
            .expect("custom series should be queryable at height");

        assert_eq!(entry.plugin().id(), ID);
        let _read = entry
            .plugin()
            .gate()
            .read_for(Duration::from_secs(1))
            .ok_or(Error::StateUpdating)?;
        let mut json = Vec::new();
        entry.vec().write_json(None, None, &mut json)?;
        Ok(json)
    }
}

impl ComputePluginSet for TestPlugins {
    fn compute(&mut self, context: UpdateContext<'_>) -> Result<()> {
        self.computed_while_closed = self.near_full_blocks.gate().try_read().is_none();
        self.near_full_blocks.compute(
            Dependencies {
                safe_height: self.safe_height,
                block_weights: &self.block_weights,
            },
            context,
        )
    }
}

#[test]
fn plugin_survives_import_publish_query_and_same_length_reorg() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let exit = Exit::new();
    let update_context = UpdateContext::new(&exit);
    let mut plugins = TestPlugins::import(directory.path())?;

    plugins.replace_tail(
        Height::ZERO,
        &[3_600_000, 3_600_001, 3_599_999, 3_600_000, 3_600_001],
    )?;
    update(&mut plugins, update_context)?;

    assert!(plugins.computed_while_closed);
    assert!(plugins.near_full_blocks.gate().try_read().is_some());
    assert_eq!(plugins.queried_streak()?, b"[1,2,0,1,2]");

    drop(plugins);
    let mut plugins = TestPlugins::import(directory.path())?;
    assert_eq!(plugins.queried_streak()?, b"[1,2,0,1,2]");

    plugins.replace_tail(Height::new(3), &[3_600_000, 3_599_999])?;
    update(&mut plugins, update_context)?;

    assert!(plugins.computed_while_closed);
    assert!(plugins.near_full_blocks.gate().try_read().is_some());
    assert_eq!(plugins.queried_streak()?, b"[1,2,0,1,0]");
    Ok(())
}
