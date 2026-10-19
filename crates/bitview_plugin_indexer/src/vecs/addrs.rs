use bitview_primitives::{
    AddrHash, P2AAddressIndex, P2PK33AddressIndex, P2PK65AddressIndex, P2PKHAddressIndex,
    P2SHAddressIndex, P2TRAddressIndex, P2WPKHAddressIndex, P2WSHAddressIndex, TypeIndex,
};
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::{
    AddressBytes, Height, OutputType, P2ABytes, P2PK33Bytes, P2PK65Bytes, P2PKHBytes, P2SHBytes,
    P2TRBytes, P2WPKHBytes, P2WSHBytes, Version,
};
use vecdb::{AnyStoredVec, Database, Rw, StorageMode, WritableVec};

use crate::readers::AddrReaders;

pub mod addr_type;

pub use addr_type::AddrTypeVecs;

#[derive(Traversable)]
pub struct AddrsVecs<M: StorageMode = Rw> {
    /// P2PK-shaped outputs containing a 65-byte key field; the field is not
    /// required to be a valid uncompressed public key.
    pub p2pk65: AddrTypeVecs<P2PK65AddressIndex, P2PK65Bytes, M>,
    /// P2PK-shaped outputs containing a 33-byte key field; the field is not
    /// required to be a valid compressed public key.
    pub p2pk33: AddrTypeVecs<P2PK33AddressIndex, P2PK33Bytes, M>,
    /// Pay-to-public-key-hash outputs.
    pub p2pkh: AddrTypeVecs<P2PKHAddressIndex, P2PKHBytes, M>,
    /// Pay-to-script-hash outputs.
    pub p2sh: AddrTypeVecs<P2SHAddressIndex, P2SHBytes, M>,
    /// Version-0 pay-to-witness-public-key-hash outputs.
    pub p2wpkh: AddrTypeVecs<P2WPKHAddressIndex, P2WPKHBytes, M>,
    /// Version-0 pay-to-witness-script-hash outputs.
    pub p2wsh: AddrTypeVecs<P2WSHAddressIndex, P2WSHBytes, M>,
    /// Pay-to-Taproot outputs.
    pub p2tr: AddrTypeVecs<P2TRAddressIndex, P2TRBytes, M>,
    /// Pay-to-Anchor outputs matching `OP_1 PUSHBYTES_2 0x4e73`.
    pub p2a: AddrTypeVecs<P2AAddressIndex, P2ABytes, M>,
}

macro_rules! addrs_vecs {
    ($(($variant:ident, $field:ident, $length:ident, $index:path, $bytes:path)),+ $(,)?) => {
        impl AddrsVecs {
            pub fn import(db: &Database, version: Version) -> Result<Self> {
                let ($($field,)+) = parallel_import! {
                    $($field = AddrTypeVecs::import(db, stringify!($field), version),)+
                };
                Ok(Self { $($field),+ })
            }

            pub fn iter_any(&self) -> impl Iterator<Item = &dyn AnyStoredVec> {
                [$(self.$field.stored_vecs()),+].into_iter().flatten()
            }

            /// Address bytes by output type, through that type's cached reader; `None` past the
            /// stored addresses.
            pub fn bytes(
                &self,
                addr_type: OutputType,
                type_index: TypeIndex,
                readers: &AddrReaders,
            ) -> Option<AddressBytes> {
                match addr_type {
                    $(OutputType::$variant => self
                        .$field
                        .bytes
                        .get_append_only(type_index.into(), &readers.$field)
                        .map(AddressBytes::from),)+
                    _ => unreachable!("address bytes of a non-address output type"),
                }
            }

            pub fn push_bytes(&mut self, index: TypeIndex, bytes: AddressBytes) {
                match bytes {
                    $(AddressBytes::$variant(bytes) => {
                        self.$field.bytes.debug_checked_push(index.into(), bytes)
                    })+
                }
            }

            /// Calls `f` with the hash of every `addr_type` address first seen at or after
            /// `height` (for rollback).
            pub fn for_each_hash_from(
                &self,
                addr_type: OutputType,
                height: Height,
                f: impl FnMut(AddrHash),
            ) {
                match addr_type {
                    $(OutputType::$variant => self.$field.for_each_hash_from(height, f),)+
                    _ => {}
                }
            }
        }
    };
}

bitview_primitives::with_address_storage_types!(addrs_vecs);
