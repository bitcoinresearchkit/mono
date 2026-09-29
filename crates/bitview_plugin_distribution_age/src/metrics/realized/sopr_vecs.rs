use bitview_cohort::{CohortContext, CohortId, UTXOGroupsWithoutAmountOrType};
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_transforms::SoprRatio;
use bitview_traversable::Traversable;
use bitview_vecs::PerBlock;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Height, StoredF32, Version};
use vecdb::{AnyStoredVec, Database, Rw, StorageMode};

use super::Sopr24hInput;

#[derive(Traversable)]
pub struct Sopr24hVecs<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub cohorts: UTXOGroupsWithoutAmountOrType<PerBlock<StoredF32, M>>,
}

impl Sopr24hVecs {
    pub fn forced_import(db: &Database, version: Version, mappings: &MappingsVecs) -> Result<Self> {
        let cohorts = UTXOGroupsWithoutAmountOrType::try_new(|cohort_id| {
            let name = CohortContext::Utxo.metric_name(cohort_id, "sopr_24h");
            let version = version
                + Version::new(3)
                + if cohort_id == CohortId::All {
                    Version::ONE
                } else {
                    Version::ZERO
                };
            PerBlock::forced_import(db, &name, version, mappings)
        })?;
        Ok(Self { cohorts })
    }

    pub fn compute(
        &mut self,
        max_from: Height,
        inputs: &UTXOGroupsWithoutAmountOrType<Sopr24hInput>,
        exit: &Exit,
    ) -> Result<()> {
        for (target, input) in self.cohorts.iter_mut().zip(inputs.iter()) {
            target.compute_binary::<_, _, SoprRatio>(
                max_from,
                &input.transfer_volume,
                &input.value_destroyed,
                exit,
            )?;
        }
        Ok(())
    }

    pub fn collect_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        self.cohorts
            .iter_mut()
            .map(|v| &mut v.height as &mut dyn AnyStoredVec)
            .collect()
    }
}
