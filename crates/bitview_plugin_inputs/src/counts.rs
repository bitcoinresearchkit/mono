use std::ops::Range;

use bitview_compute::write_locked;
use bitview_plugin_indexer::{BlockCounterVecs, Indexer};
use bitview_primitives::{Bytes32, Count};
use bitview_traversable::Traversable;
use bitview_vecs::{
    CumulativeRolling, IndexSources, LazyWindowStartVec, PerBlockCumulativeRolling, counter_group,
    shapes::Windows,
};
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Height, Version, Weight32};
use vecdb::{AnyVec, Database, ReadableVec, Rw, StorageMode, VecIndex};

/// What inputs reveal when they spend: signatures, keys, scripts, how old the coins are.
#[derive(Traversable)]
pub(crate) struct Counts<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub(crate) revealed: Revealed<M>,
    #[traversable(flatten)]
    pub(crate) ages: SpendAges<M>,
    /// Coinbase transactions with a witness.
    #[traversable(lead = "Coinbase inputs with a witness.")]
    pub(crate) coinbase_witness: CumulativeRolling<Count, M>,
}

impl Counts {
    pub(crate) fn import(
        db: &Database,
        version: Version,
        indexes: &IndexSources,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        Ok(Self {
            revealed: Revealed::import(db, version, indexes, window_starts)?,
            ages: SpendAges::import(db, version, indexes, window_starts)?,
            coinbase_witness: CumulativeRolling::import(
                db,
                "coinbase_witness_block_count",
                version,
                indexes,
                window_starts,
            )?,
        })
    }
}

#[derive(Traversable)]
pub(crate) struct Revealed<M: StorageMode = Rw> {
    signatures: InputSignatures<M>,
    pubkeys: Pubkeys<M>,
    nested_segwit: NestedSegwit<M>,
    multisig: Multisig<M>,
    opcodes: Opcodes<M>,
    #[traversable(flatten)]
    inputs: Inputs<M>,
}

impl Revealed {
    fn import(
        db: &Database,
        version: Version,
        indexes: &IndexSources,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        Ok(Self {
            signatures: InputSignatures::import(db, version, indexes, window_starts)?,
            pubkeys: Pubkeys::import(db, version, indexes, window_starts)?,
            nested_segwit: NestedSegwit::import(db, version, indexes, window_starts)?,
            multisig: Multisig::import(db, version, indexes, window_starts)?,
            opcodes: Opcodes::import(db, version, indexes, window_starts)?,
            inputs: Inputs::import(db, version, indexes, window_starts)?,
        })
    }

    pub(crate) fn compute(
        &mut self,
        max_from: Height,
        counters: &BlockCounterVecs,
        exit: &Exit,
    ) -> Result<()> {
        self.signatures.compute(max_from, counters, exit)?;
        self.pubkeys.compute(max_from, counters, exit)?;
        self.nested_segwit.compute(max_from, counters, exit)?;
        self.multisig.compute(max_from, counters, exit)?;
        self.opcodes.compute(max_from, counters, exit)?;
        self.inputs.compute(max_from, counters, exit)
    }
}

#[derive(Traversable)]
struct InputSignatures<M: StorageMode = Rw> {
    schnorr: SchnorrSignatures<M>,
    ecdsa: EcdsaSignatures<M>,
}

impl InputSignatures {
    fn import(
        db: &Database,
        version: Version,
        indexes: &IndexSources,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        Ok(Self {
            schnorr: SchnorrSignatures::import(db, version, indexes, window_starts)?,
            ecdsa: EcdsaSignatures::import(db, version, indexes, window_starts)?,
        })
    }

    fn compute(
        &mut self,
        max_from: Height,
        counters: &BlockCounterVecs,
        exit: &Exit,
    ) -> Result<()> {
        self.schnorr.compute(max_from, counters, exit)?;
        self.ecdsa.compute(max_from, counters, exit)
    }
}

counter_group! {
    BlockCounterVecs;

    struct SchnorrSignatures {
        /// Taproot signatures: 64 bytes, or 65 whose last byte names a sighash type other than
        /// the default, in a key-path spend's witness or a tapscript spend's arguments.
        #[traversable(lead = "Schnorr signatures in inputs.")]
        count: Count => schnorr_signature_count,
        /// The bytes of those signatures, sighash bytes included.
        #[traversable(lead = "Bytes of Schnorr signatures in inputs.")]
        bytes: Bytes => schnorr_signature_bytes,
    }

    struct EcdsaTotals {
        /// ECDSA signatures in inputs: strict DER, or a lax encoding secp256k1 parses, in the
        /// places each spent type carries them.
        #[traversable(lead = "ECDSA signatures in inputs.")]
        count: Count => ecdsa_signature_count,
        /// The bytes of those signatures, sighash bytes included.
        #[traversable(lead = "Bytes of ECDSA signatures in inputs.")]
        bytes: Bytes => ecdsa_signature_bytes,
        /// ECDSA signatures in strict DER, the encoding BIP-66 requires.
        #[traversable(lead = "ECDSA signatures in strict DER.")]
        strict_der: Count => strict_der_ecdsa_signature_count,
        /// ECDSA signatures whose r is below 2^255.
        #[traversable(lead = "ECDSA signatures with a low r.")]
        low_r: Count => low_r_ecdsa_signature_count,
        /// ECDSA signatures whose s is at most half the curve order.
        #[traversable(lead = "ECDSA signatures with a low s.")]
        low_s: Count => low_s_ecdsa_signature_count,
        /// ECDSA signatures with both a low r and a low s.
        #[traversable(lead = "ECDSA signatures with both r and s low.")]
        low_r_low_s: Count => low_r_low_s_ecdsa_signature_count,
    }

    struct EcdsaLengths {
        #[traversable(lead = "ECDSA signatures of 70 bytes.")]
        _70_bytes: Count => ecdsa_70_bytes_signature_count,
        #[traversable(lead = "ECDSA signatures of 71 bytes.")]
        _71_bytes: Count => ecdsa_71_bytes_signature_count,
        #[traversable(lead = "ECDSA signatures of 72 bytes.")]
        _72_bytes: Count => ecdsa_72_bytes_signature_count,
        #[traversable(lead = "ECDSA signatures of 73 bytes.")]
        _73_bytes: Count => ecdsa_73_bytes_signature_count,
        #[traversable(lead = "ECDSA signatures of 74 bytes.")]
        _74_bytes: Count => ecdsa_74_bytes_signature_count,
        #[traversable(lead = "ECDSA signatures of 75 bytes or more.")]
        over_74_bytes: Count => ecdsa_over_74_bytes_signature_count,
    }

    struct EcdsaSighashes {
        #[traversable(lead = "ECDSA signatures with SIGHASH_ALL.")]
        all: Count => sighash_all_ecdsa_signature_count,
        #[traversable(lead = "ECDSA signatures with SIGHASH_NONE.")]
        none: Count => sighash_none_ecdsa_signature_count,
        #[traversable(lead = "ECDSA signatures with SIGHASH_SINGLE.")]
        single: Count => sighash_single_ecdsa_signature_count,
        #[traversable(lead = "ECDSA signatures with SIGHASH_ALL and ANYONECANPAY.")]
        all_anyone_can_pay: Count => sighash_all_anyone_can_pay_ecdsa_signature_count,
        #[traversable(lead = "ECDSA signatures with SIGHASH_NONE and ANYONECANPAY.")]
        none_anyone_can_pay: Count => sighash_none_anyone_can_pay_ecdsa_signature_count,
        #[traversable(lead = "ECDSA signatures with SIGHASH_SINGLE and ANYONECANPAY.")]
        single_anyone_can_pay: Count => sighash_single_anyone_can_pay_ecdsa_signature_count,
    }

    struct Pubkeys {
        /// 33-byte public keys that inputs reveal: a spend's own key, or the keys in a redeem or
        /// witness script. By shape: a key in a script isn't checked to be on the curve.
        #[traversable(lead = "Compressed public keys in inputs.")]
        compressed: Count => compressed_pubkey_in_input_count,
        /// 65-byte public keys that inputs reveal, by the same rule.
        #[traversable(lead = "Uncompressed public keys in inputs.")]
        uncompressed: Count => uncompressed_pubkey_in_input_count,
    }

    struct NestedSegwit {
        /// P2SH inputs carrying a version-0 P2WPKH program.
        #[traversable(lead = "P2SH-wrapped P2WPKH inputs.")]
        p2wpkh: Count => p2sh_p2wpkh_input_count,
        /// P2SH inputs carrying a version-0 P2WSH program.
        #[traversable(lead = "P2SH-wrapped P2WSH inputs.")]
        p2wsh: Count => p2sh_p2wsh_input_count,
    }

    struct Multisig {
        /// P2SH inputs whose redeem script is `OP_n <m keys> OP_m OP_CHECKMULTISIG`.
        #[traversable(lead = "Multisig P2SH inputs.")]
        p2sh: Count => p2sh_multisig_input_count,
        /// Nested P2WSH inputs whose witness script has that form.
        #[traversable(lead = "Multisig nested P2WSH inputs.")]
        p2sh_p2wsh: Count => p2sh_p2wsh_multisig_input_count,
        /// P2WSH inputs whose witness script has that form.
        #[traversable(lead = "Multisig P2WSH inputs.")]
        p2wsh: Count => p2wsh_multisig_input_count,
    }

    /// Occurrences of each opcode in the scripts inputs reveal: tapscripts, witness scripts and
    /// redeem scripts. The fixed templates of P2PKH and P2WPKH spends aren't counted.
    struct Opcodes {
        #[traversable(lead = "OP_CHECKSIG in revealed scripts.")]
        checksig: Count => op_checksig_in_input_count,
        #[traversable(lead = "OP_CHECKSIGVERIFY in revealed scripts.")]
        checksigverify: Count => op_checksigverify_in_input_count,
        #[traversable(lead = "OP_CHECKMULTISIG in revealed scripts.")]
        checkmultisig: Count => op_checkmultisig_in_input_count,
        #[traversable(lead = "OP_CHECKSIGADD in revealed scripts.")]
        checksigadd: Count => op_checksigadd_in_input_count,
        #[traversable(lead = "OP_CHECKLOCKTIMEVERIFY in revealed scripts.")]
        checklocktimeverify: Count => op_checklocktimeverify_in_input_count,
        #[traversable(lead = "OP_CHECKSEQUENCEVERIFY in revealed scripts.")]
        checksequenceverify: Count => op_checksequenceverify_in_input_count,
        #[traversable(lead = "OP_HASH160 in revealed scripts.")]
        hash160: Count => op_hash160_in_input_count,
        #[traversable(lead = "OP_SHA256 in revealed scripts.")]
        sha256: Count => op_sha256_in_input_count,
        #[traversable(lead = "OP_IF in revealed scripts.")]
        r#if: Count => op_if_in_input_count,
        #[traversable(lead = "OP_NOTIF in revealed scripts.")]
        notif: Count => op_notif_in_input_count,
    }

    struct Inputs {
        /// P2TR inputs spent by a script path: key-path inputs are all P2TR inputs less these.
        #[traversable(lead = "Taproot script-path inputs.")]
        p2tr_script_path: Count => p2tr_script_path_input_count,
        /// Inputs revealing an inscription envelope in their tapscript.
        #[traversable(lead = "Inputs revealing an inscription.")]
        inscriptions: Count => inscription_input_count,
    }
}

#[derive(Traversable)]
struct EcdsaSignatures<M: StorageMode = Rw> {
    #[traversable(flatten)]
    totals: EcdsaTotals<M>,
    /// By the signature's length in bytes, sighash byte included.
    length: EcdsaLengths<M>,
    /// By the sighash byte.
    sighash: EcdsaSighashes<M>,
}

impl EcdsaSignatures {
    fn import(
        db: &Database,
        version: Version,
        indexes: &IndexSources,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        Ok(Self {
            totals: EcdsaTotals::import(db, version, indexes, window_starts)?,
            length: EcdsaLengths::import(db, version, indexes, window_starts)?,
            sighash: EcdsaSighashes::import(db, version, indexes, window_starts)?,
        })
    }

    fn compute(
        &mut self,
        max_from: Height,
        counters: &BlockCounterVecs,
        exit: &Exit,
    ) -> Result<()> {
        self.totals.compute(max_from, counters, exit)?;
        self.length.compute(max_from, counters, exit)?;
        self.sighash.compute(max_from, counters, exit)
    }
}

/// A coinbase transaction has a witness when its block's witness discount (four times the size
/// less the weight) isn't all its other SegWit transactions': the header, the transaction count
/// and every transaction without a witness weigh four times their size.
pub(crate) fn compute_coinbase_witness(
    total: &mut CumulativeRolling<Count>,
    indexer: &Indexer,
    from: Height,
    exit: &Exit,
) -> Result<()> {
    let blocks = &indexer.vecs().blocks;
    let (size, weight) = (&*blocks.size, &*blocks.weight);
    let (segwit_size, segwit_weight) = (&*blocks.segwit_tx_size, &*blocks.segwit_tx_weight);
    let Range { start, end } = bitview_compute::Inputs::new(
        Version::ZERO,
        [size as &dyn AnyVec, weight, segwit_size, segwit_weight],
    )
    .prepare([total.stored_mut()], from.to_usize(), exit)?;
    if start < end {
        let discount = |size: Bytes32, weight: Weight32| {
            4 * u64::from(u32::from(size)) - u64::from(u32::from(weight))
        };
        let blocks = size
            .collect_range_at(start, end)
            .into_iter()
            .zip(weight.collect_range_at(start, end))
            .zip(segwit_size.collect_range_at(start, end))
            .zip(segwit_weight.collect_range_at(start, end));
        for (((size, weight), segwit_size), segwit_weight) in blocks {
            let has_witness = discount(size, weight) != discount(segwit_size, segwit_weight);
            total.push_block(Count::from(has_witness));
        }
    }
    write_locked([total.stored_mut()], exit)
}

/// How young the coins spent were, which the spends walk fills a block at a time.
#[derive(Traversable)]
pub(crate) struct SpendAges<M: StorageMode = Rw> {
    /// Inputs by how many blocks before them the output they spend was created.
    spent_within: SpentWithin<M>,
    #[traversable(lead = "Inputs spending an output of their own block.")]
    same_block: PerBlockCumulativeRolling<Count, M>,
}

#[derive(Traversable)]
struct SpentWithin<M: StorageMode = Rw> {
    #[traversable(lead = "Inputs spending an output at most 1 block old.")]
    _1_block: PerBlockCumulativeRolling<Count, M>,
    #[traversable(lead = "Inputs spending an output at most 6 blocks old.")]
    _6_blocks: PerBlockCumulativeRolling<Count, M>,
    #[traversable(lead = "Inputs spending an output at most 144 blocks old.")]
    _144_blocks: PerBlockCumulativeRolling<Count, M>,
    #[traversable(lead = "Inputs spending an output at most 2,016 blocks old.")]
    _2016_blocks: PerBlockCumulativeRolling<Count, M>,
}

impl SpendAges {
    /// The ages `spent_within` counts up to, in blocks.
    pub(crate) const AGES: [u32; 4] = [1, 6, 144, 2016];

    fn import(
        db: &Database,
        version: Version,
        indexes: &IndexSources,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        let count = |name: &str| {
            PerBlockCumulativeRolling::import(db, name, version, indexes, window_starts)
        };
        let within = |blocks: u32| {
            let unit = if blocks == 1 { "block" } else { "blocks" };
            count(&format!("within_{blocks}_{unit}_spent_utxo_count"))
        };
        Ok(Self {
            spent_within: SpentWithin {
                _1_block: within(1)?,
                _6_blocks: within(6)?,
                _144_blocks: within(144)?,
                _2016_blocks: within(2016)?,
            },
            same_block: count("same_block_spent_utxo_count")?,
        })
    }

    /// Block `height`'s, from its inputs counted by the height of the output each spends: those
    /// at most each of [`Self::AGES`] old, and those spending an output of the block itself.
    pub(crate) fn push_block(&mut self, height: u32, spent: impl Iterator<Item = (u32, u64)>) {
        let mut within = [0u64; Self::AGES.len()];
        let mut same_block = 0;
        for (origin, count) in spent {
            let age = height - origin;
            for (within, limit) in within.iter_mut().zip(Self::AGES) {
                *within += if age <= limit { count } else { 0 };
            }
            if age == 0 {
                same_block = count;
            }
        }
        let w = &mut self.spent_within;
        for (total, count) in [
            &mut w._1_block,
            &mut w._6_blocks,
            &mut w._144_blocks,
            &mut w._2016_blocks,
        ]
        .into_iter()
        .chain([&mut self.same_block])
        .zip(within.into_iter().chain([same_block]))
        {
            total.push_block(Count::from(count));
        }
    }
}
