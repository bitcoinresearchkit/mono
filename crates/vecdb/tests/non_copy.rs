use tempfile::TempDir;
#[cfg(feature = "lz4")]
use vecdb::LZ4Vec;
use vecdb::{Bytes, BytesVec, Database, Error, ReadableVec, Result, StoredVec, Version};

#[derive(Debug, Clone, PartialEq, Eq)]
struct HeapValue(Box<u64>);

impl Bytes for HeapValue {
    type Array = [u8; 8];

    fn to_bytes(&self) -> Self::Array {
        self.0.to_le_bytes()
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let bytes: [u8; 8] = bytes.try_into().map_err(|_| Error::WrongLength {
            expected: 8,
            received: bytes.len(),
        })?;
        Ok(Self(Box::new(u64::from_le_bytes(bytes))))
    }
}

fn check_non_copy_reads<V: StoredVec<I = usize, T = HeapValue>>() -> Result<()> {
    let temp = TempDir::new()?;
    let db = Database::open(temp.path())?;
    let mut vec = V::import(&db, "heap", Version::ONE)?;

    for value in 0..5_000 {
        vec.push(HeapValue(Box::new(value)));
    }
    vec.write()?;

    let sum = vec.fold(0_u64, |sum, value| sum + *value.0);
    assert_eq!(sum, (0..5_000_u64).sum::<u64>());
    let reader = vec.read_only_boxed_clone();
    let mut values = Vec::new();
    reader.for_each_chunk_at(17, usize::MAX, &mut |at, chunk| {
        assert_eq!(at, 17 + values.len());
        values.extend(chunk.iter().map(|value| *value.0));
    });
    assert_eq!(values, (17..5_000).collect::<Vec<_>>());

    Ok(())
}

#[cfg(feature = "lz4")]
#[test]
fn compressed_fold_clones_non_copy_values() -> Result<()> {
    check_non_copy_reads::<LZ4Vec<usize, HeapValue>>()
}

#[test]
fn raw_chunks_decode_non_native_values() -> Result<()> {
    check_non_copy_reads::<BytesVec<usize, HeapValue>>()
}
