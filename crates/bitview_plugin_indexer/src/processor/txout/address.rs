mod cache;

use brk_error::Result;

use std::collections::hash_map::Entry;

use bitview_cohort::ByAddrType;
use brk_error::Error;
use brk_types::{AddrBytes, AddrHash, OutputType, TxIndex, TypeIndex, Vout};
use rayon::prelude::*;
use rustc_hash::FxHashMap;
use tracing::error;
use vecdb::likely;

use self::cache::AddressCache;
use super::{ProcessedOutput, processed::ProcessedOutputData};
use crate::processor::BlockProcessor;

pub struct Lookup {
    index: usize,
    output_type: OutputType,
    hash: AddrHash,
    type_index: Option<TypeIndex>,
}

#[derive(Default)]
pub struct BlockAddresses {
    cache: AddressCache,
    indexes: ByAddrType<FxHashMap<AddrHash, usize>>,
    lookups: Vec<Lookup>,
    unique: Vec<(OutputType, AddrHash)>,
    resolved: Vec<Option<TypeIndex>>,
}

impl BlockAddresses {
    pub fn resolve(
        &mut self,
        processor: &BlockProcessor,
        outputs: &[ProcessedOutput],
    ) -> Result<()> {
        self.clear_block();

        for output in outputs {
            let ProcessedOutputData::Address(addr_hash) = &output.data else {
                continue;
            };

            if let Entry::Vacant(entry) = self
                .indexes
                .get_mut_unwrap(output.output_type)
                .entry(*addr_hash)
            {
                entry.insert(self.unique.len());
                self.unique.push((output.output_type, *addr_hash));
            }
        }

        self.resolved.resize(self.unique.len(), None);

        for (index, &(output_type, hash)) in self.unique.iter().enumerate() {
            if let Some(type_index) = self.cache.get(output_type, hash) {
                self.resolved[index] = Some(type_index);
            } else {
                self.lookups.push(Lookup {
                    index,
                    output_type,
                    hash,
                    type_index: None,
                });
            }
        }

        self.lookups
            .sort_unstable_by_key(|lookup| (lookup.output_type, lookup.hash));

        let lengths = &*processor.lengths;

        self.lookups
            .par_iter_mut()
            .try_for_each(|lookup| -> Result<()> {
                lookup.type_index = processor
                    .stores
                    .addr_index(lookup.output_type, &lookup.hash)?
                    .filter(|type_index| *type_index < lengths.to_type_index(lookup.output_type));
                Ok(())
            })?;

        for lookup in &self.lookups {
            self.resolved[lookup.index] = lookup.type_index;
            if let Some(type_index) = lookup.type_index {
                self.cache
                    .insert(lookup.output_type, lookup.hash, type_index);
            }
        }

        if likely(!processor.check_collisions) {
            return Ok(());
        }

        let mut output_offset = 0;
        for (block_tx_index, tx) in processor.block.txdata.iter().enumerate() {
            let tx_index = processor.lengths.tx_index + TxIndex::from(block_tx_index);
            let next_output_offset = output_offset + tx.output.len();

            for (vout, (txout, output)) in tx
                .output
                .iter()
                .zip(&outputs[output_offset..next_output_offset])
                .enumerate()
            {
                let ProcessedOutputData::Address(addr_hash) = &output.data else {
                    continue;
                };
                let Some(type_index) = self.index(output.output_type, addr_hash) else {
                    continue;
                };
                let addr_bytes =
                    AddrBytes::try_from((&txout.script_pubkey, output.output_type)).unwrap();

                let prev_addrbytes = processor
                    .vecs
                    .addrs
                    .get_bytes_by_type(output.output_type, type_index, &processor.readers.addrbytes)
                    .ok_or(Error::Internal("Missing addrbytes"))?;

                if prev_addrbytes != addr_bytes {
                    error!(
                        height = ?processor.height,
                        vout = ?Vout::from(vout),
                        ?tx_index,
                        addr_type = ?output.output_type,
                        ?prev_addrbytes,
                        ?addr_bytes,
                        ?type_index,
                        "Address hash collision"
                    );
                    return Err(Error::Internal("Address hash collision"));
                }
            }

            output_offset = next_output_offset;
        }

        Ok(())
    }

    pub fn index_mut(
        &mut self,
        addr_type: OutputType,
        addr_hash: &AddrHash,
    ) -> &mut Option<TypeIndex> {
        let index = *self
            .indexes
            .get_mut_unwrap(addr_type)
            .get_mut(addr_hash)
            .unwrap();
        &mut self.resolved[index]
    }

    pub fn index(&self, addr_type: OutputType, addr_hash: &AddrHash) -> Option<TypeIndex> {
        let index = *self.indexes.get_unwrap(addr_type).get(addr_hash).unwrap();
        self.resolved[index]
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn clear_block(&mut self) {
        self.indexes.values_mut().for_each(FxHashMap::clear);
        self.lookups.clear();
        self.unique.clear();
        self.resolved.clear();
    }
}
