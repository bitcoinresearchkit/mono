mod addr_type;

pub use addr_type::AddrTypeVecs;

use brk_error::Result;
use brk_traversable::Traversable;
use brk_types::{
    AddrBytes, AddrHash, Height, OutputType, P2AAddrIndex, P2ABytes, P2PK33AddrIndex, P2PK33Bytes,
    P2PK65AddrIndex, P2PK65Bytes, P2PKHAddrIndex, P2PKHBytes, P2SHAddrIndex, P2SHBytes,
    P2TRAddrIndex, P2TRBytes, P2WPKHAddrIndex, P2WPKHBytes, P2WSHAddrIndex, P2WSHBytes, TypeIndex,
    Version,
};
use rayon::prelude::*;
use vecdb::{
    AnyStoredVec, BytesVec, Database, ImportableVec, PcoVec, ReadableVec, Ro, Rw, Stamp,
    StorageMode, WritableVec,
};

use crate::{parallel_import, readers::AddrReaders};

#[derive(Traversable)]
pub struct AddrsVecs<M: StorageMode = Rw> {
    /// P2PK-shaped outputs containing a 65-byte key field; the field is not
    /// required to be a valid uncompressed public key.
    pub p2pk65: AddrTypeVecs<P2PK65AddrIndex, P2PK65Bytes, M>,
    /// P2PK-shaped outputs containing a 33-byte key field; the field is not
    /// required to be a valid compressed public key.
    pub p2pk33: AddrTypeVecs<P2PK33AddrIndex, P2PK33Bytes, M>,
    /// Pay-to-public-key-hash outputs.
    pub p2pkh: AddrTypeVecs<P2PKHAddrIndex, P2PKHBytes, M>,
    /// Pay-to-script-hash outputs.
    pub p2sh: AddrTypeVecs<P2SHAddrIndex, P2SHBytes, M>,
    /// Version-0 pay-to-witness-public-key-hash outputs.
    pub p2wpkh: AddrTypeVecs<P2WPKHAddrIndex, P2WPKHBytes, M>,
    /// Version-0 pay-to-witness-script-hash outputs.
    pub p2wsh: AddrTypeVecs<P2WSHAddrIndex, P2WSHBytes, M>,
    /// Pay-to-Taproot outputs.
    pub p2tr: AddrTypeVecs<P2TRAddrIndex, P2TRBytes, M>,
    /// Pay-to-Anchor outputs matching `OP_1 PUSHBYTES_2 0x4e73`.
    pub p2a: AddrTypeVecs<P2AAddrIndex, P2ABytes, M>,
}

impl AddrsVecs {
    pub fn forced_import(db: &Database, version: Version) -> Result<Self> {
        let (
            first_p2pk65_addr_index,
            first_p2pk33_addr_index,
            first_p2pkh_addr_index,
            first_p2sh_addr_index,
            first_p2wpkh_addr_index,
            first_p2wsh_addr_index,
            first_p2tr_addr_index,
            first_p2a_addr_index,
            p2pk65_bytes,
            p2pk33_bytes,
            p2pkh_bytes,
            p2sh_bytes,
            p2wpkh_bytes,
            p2wsh_bytes,
            p2tr_bytes,
            p2a_bytes,
        ) = parallel_import! {
            first_p2pk65_addr_index = PcoVec::forced_import(db, "first_p2pk65_addr_index", version),
            first_p2pk33_addr_index = PcoVec::forced_import(db, "first_p2pk33_addr_index", version),
            first_p2pkh_addr_index = PcoVec::forced_import(db, "first_p2pkh_addr_index", version),
            first_p2sh_addr_index = PcoVec::forced_import(db, "first_p2sh_addr_index", version),
            first_p2wpkh_addr_index = PcoVec::forced_import(db, "first_p2wpkh_addr_index", version),
            first_p2wsh_addr_index = PcoVec::forced_import(db, "first_p2wsh_addr_index", version),
            first_p2tr_addr_index = PcoVec::forced_import(db, "first_p2tr_addr_index", version),
            first_p2a_addr_index = PcoVec::forced_import(db, "first_p2a_addr_index", version),
            p2pk65_bytes = BytesVec::forced_import(db, "p2pk65_bytes", version),
            p2pk33_bytes = BytesVec::forced_import(db, "p2pk33_bytes", version),
            p2pkh_bytes = BytesVec::forced_import(db, "p2pkh_bytes", version),
            p2sh_bytes = BytesVec::forced_import(db, "p2sh_bytes", version),
            p2wpkh_bytes = BytesVec::forced_import(db, "p2wpkh_bytes", version),
            p2wsh_bytes = BytesVec::forced_import(db, "p2wsh_bytes", version),
            p2tr_bytes = BytesVec::forced_import(db, "p2tr_bytes", version),
            p2a_bytes = BytesVec::forced_import(db, "p2a_bytes", version),
        };
        Ok(Self {
            p2pk65: AddrTypeVecs {
                first_index: first_p2pk65_addr_index,
                bytes: p2pk65_bytes,
            },
            p2pk33: AddrTypeVecs {
                first_index: first_p2pk33_addr_index,
                bytes: p2pk33_bytes,
            },
            p2pkh: AddrTypeVecs {
                first_index: first_p2pkh_addr_index,
                bytes: p2pkh_bytes,
            },
            p2sh: AddrTypeVecs {
                first_index: first_p2sh_addr_index,
                bytes: p2sh_bytes,
            },
            p2wpkh: AddrTypeVecs {
                first_index: first_p2wpkh_addr_index,
                bytes: p2wpkh_bytes,
            },
            p2wsh: AddrTypeVecs {
                first_index: first_p2wsh_addr_index,
                bytes: p2wsh_bytes,
            },
            p2tr: AddrTypeVecs {
                first_index: first_p2tr_addr_index,
                bytes: p2tr_bytes,
            },
            p2a: AddrTypeVecs {
                first_index: first_p2a_addr_index,
                bytes: p2a_bytes,
            },
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn truncate(
        &mut self,
        height: Height,
        p2pk65_addr_index: P2PK65AddrIndex,
        p2pk33_addr_index: P2PK33AddrIndex,
        p2pkh_addr_index: P2PKHAddrIndex,
        p2sh_addr_index: P2SHAddrIndex,
        p2wpkh_addr_index: P2WPKHAddrIndex,
        p2wsh_addr_index: P2WSHAddrIndex,
        p2tr_addr_index: P2TRAddrIndex,
        p2a_addr_index: P2AAddrIndex,
        stamp: Stamp,
    ) -> Result<()> {
        self.p2pk65
            .first_index
            .truncate_if_needed_with_stamp(height, stamp)?;
        self.p2pk33
            .first_index
            .truncate_if_needed_with_stamp(height, stamp)?;
        self.p2pkh
            .first_index
            .truncate_if_needed_with_stamp(height, stamp)?;
        self.p2sh
            .first_index
            .truncate_if_needed_with_stamp(height, stamp)?;
        self.p2wpkh
            .first_index
            .truncate_if_needed_with_stamp(height, stamp)?;
        self.p2wsh
            .first_index
            .truncate_if_needed_with_stamp(height, stamp)?;
        self.p2tr
            .first_index
            .truncate_if_needed_with_stamp(height, stamp)?;
        self.p2a
            .first_index
            .truncate_if_needed_with_stamp(height, stamp)?;
        self.p2pk65
            .bytes
            .truncate_if_needed_with_stamp(p2pk65_addr_index, stamp)?;
        self.p2pk33
            .bytes
            .truncate_if_needed_with_stamp(p2pk33_addr_index, stamp)?;
        self.p2pkh
            .bytes
            .truncate_if_needed_with_stamp(p2pkh_addr_index, stamp)?;
        self.p2sh
            .bytes
            .truncate_if_needed_with_stamp(p2sh_addr_index, stamp)?;
        self.p2wpkh
            .bytes
            .truncate_if_needed_with_stamp(p2wpkh_addr_index, stamp)?;
        self.p2wsh
            .bytes
            .truncate_if_needed_with_stamp(p2wsh_addr_index, stamp)?;
        self.p2tr
            .bytes
            .truncate_if_needed_with_stamp(p2tr_addr_index, stamp)?;
        self.p2a
            .bytes
            .truncate_if_needed_with_stamp(p2a_addr_index, stamp)?;
        Ok(())
    }

    pub fn par_iter_mut_any(&mut self) -> impl ParallelIterator<Item = &mut dyn AnyStoredVec> {
        [
            &mut self.p2pk65.first_index as &mut dyn AnyStoredVec,
            &mut self.p2pk33.first_index,
            &mut self.p2pkh.first_index,
            &mut self.p2sh.first_index,
            &mut self.p2wpkh.first_index,
            &mut self.p2wsh.first_index,
            &mut self.p2tr.first_index,
            &mut self.p2a.first_index,
            &mut self.p2pk65.bytes,
            &mut self.p2pk33.bytes,
            &mut self.p2pkh.bytes,
            &mut self.p2sh.bytes,
            &mut self.p2wpkh.bytes,
            &mut self.p2wsh.bytes,
            &mut self.p2tr.bytes,
            &mut self.p2a.bytes,
        ]
        .into_par_iter()
    }

    pub fn iter_any(&self) -> impl Iterator<Item = &dyn AnyStoredVec> {
        [
            &self.p2pk65.first_index as &dyn AnyStoredVec,
            &self.p2pk33.first_index,
            &self.p2pkh.first_index,
            &self.p2sh.first_index,
            &self.p2wpkh.first_index,
            &self.p2wsh.first_index,
            &self.p2tr.first_index,
            &self.p2a.first_index,
            &self.p2pk65.bytes,
            &self.p2pk33.bytes,
            &self.p2pkh.bytes,
            &self.p2sh.bytes,
            &self.p2wpkh.bytes,
            &self.p2wsh.bytes,
            &self.p2tr.bytes,
            &self.p2a.bytes,
        ]
        .into_iter()
    }

    /// Get address bytes by output type, using the cached VecReader for the specific address type.
    /// Returns None if the index doesn't exist yet.
    pub fn get_bytes_by_type(
        &self,
        addr_type: OutputType,
        type_index: TypeIndex,
        readers: &AddrReaders,
    ) -> Option<AddrBytes> {
        match addr_type {
            OutputType::P2PK65 => self
                .p2pk65
                .bytes
                .get_append_only(type_index.into(), &readers.p2pk65)
                .map(AddrBytes::from),
            OutputType::P2PK33 => self
                .p2pk33
                .bytes
                .get_append_only(type_index.into(), &readers.p2pk33)
                .map(AddrBytes::from),
            OutputType::P2PKH => self
                .p2pkh
                .bytes
                .get_append_only(type_index.into(), &readers.p2pkh)
                .map(AddrBytes::from),
            OutputType::P2SH => self
                .p2sh
                .bytes
                .get_append_only(type_index.into(), &readers.p2sh)
                .map(AddrBytes::from),
            OutputType::P2WPKH => self
                .p2wpkh
                .bytes
                .get_append_only(type_index.into(), &readers.p2wpkh)
                .map(AddrBytes::from),
            OutputType::P2WSH => self
                .p2wsh
                .bytes
                .get_append_only(type_index.into(), &readers.p2wsh)
                .map(AddrBytes::from),
            OutputType::P2TR => self
                .p2tr
                .bytes
                .get_append_only(type_index.into(), &readers.p2tr)
                .map(AddrBytes::from),
            OutputType::P2A => self
                .p2a
                .bytes
                .get_append_only(type_index.into(), &readers.p2a)
                .map(AddrBytes::from),
            _ => unreachable!("get_bytes_by_type called with non-address type"),
        }
    }

    pub fn push_bytes_if_needed(&mut self, index: TypeIndex, bytes: AddrBytes) {
        match bytes {
            AddrBytes::P2PK65(bytes) => self.p2pk65.bytes.debug_checked_push(index.into(), bytes),
            AddrBytes::P2PK33(bytes) => self.p2pk33.bytes.debug_checked_push(index.into(), bytes),
            AddrBytes::P2PKH(bytes) => self.p2pkh.bytes.debug_checked_push(index.into(), bytes),
            AddrBytes::P2SH(bytes) => self.p2sh.bytes.debug_checked_push(index.into(), bytes),
            AddrBytes::P2WPKH(bytes) => self.p2wpkh.bytes.debug_checked_push(index.into(), bytes),
            AddrBytes::P2WSH(bytes) => self.p2wsh.bytes.debug_checked_push(index.into(), bytes),
            AddrBytes::P2TR(bytes) => self.p2tr.bytes.debug_checked_push(index.into(), bytes),
            AddrBytes::P2A(bytes) => self.p2a.bytes.debug_checked_push(index.into(), bytes),
        };
    }

    /// Iterate address hashes starting from a given height (for rollback).
    /// Returns an iterator of AddrHash values for all addresses of the given type
    /// that were added at or after the given height.
    pub fn iter_hashes_from(
        &self,
        addr_type: OutputType,
        height: Height,
    ) -> Result<Box<dyn Iterator<Item = AddrHash> + '_>> {
        macro_rules! make_iter {
            ($addr:expr) => {{
                match $addr.first_index.collect_one(height) {
                    Some(mut index) => {
                        let reader = $addr.bytes.reader();
                        Ok(Box::new(std::iter::from_fn(move || {
                            reader.try_get(index).map(|typedbytes| {
                                let bytes = AddrBytes::from(typedbytes);
                                index.increment();
                                AddrHash::from(&bytes)
                            })
                        }))
                            as Box<dyn Iterator<Item = AddrHash> + '_>)
                    }
                    None => {
                        Ok(Box::new(std::iter::empty()) as Box<dyn Iterator<Item = AddrHash> + '_>)
                    }
                }
            }};
        }

        match addr_type {
            OutputType::P2PK65 => make_iter!(self.p2pk65),
            OutputType::P2PK33 => make_iter!(self.p2pk33),
            OutputType::P2PKH => make_iter!(self.p2pkh),
            OutputType::P2SH => make_iter!(self.p2sh),
            OutputType::P2WPKH => make_iter!(self.p2wpkh),
            OutputType::P2WSH => make_iter!(self.p2wsh),
            OutputType::P2TR => make_iter!(self.p2tr),
            OutputType::P2A => make_iter!(self.p2a),
            _ => Ok(Box::new(std::iter::empty())),
        }
    }
}

macro_rules! impl_addr_readers {
    ($mode:ty) => {
        impl AddrsVecs<$mode> {
            pub fn addr_readers(&self) -> AddrReaders {
                AddrReaders {
                    p2pk65: self.p2pk65.bytes.reader(),
                    p2pk33: self.p2pk33.bytes.reader(),
                    p2pkh: self.p2pkh.bytes.reader(),
                    p2sh: self.p2sh.bytes.reader(),
                    p2wpkh: self.p2wpkh.bytes.reader(),
                    p2wsh: self.p2wsh.bytes.reader(),
                    p2tr: self.p2tr.bytes.reader(),
                    p2a: self.p2a.bytes.reader(),
                }
            }
        }
    };
}

impl_addr_readers!(Rw);
impl_addr_readers!(Ro);
