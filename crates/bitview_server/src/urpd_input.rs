use bitcoin::hashes::{Hash, HashEngine, sha256};
use bitview_query::ResolvedUrpd;
use brk_error::Result;

/// Validate captured inputs before deriving their representation identity.
pub fn identity(input: &ResolvedUrpd) -> Result<sha256::Hash> {
    input.validate()?;
    let mut engine = sha256::Hash::engine();
    engine.input(b"urpd2\0");
    engine.input(&u32::from(input.height).to_le_bytes());
    engine.input(input.cohort.as_bytes());
    engine.input(&[0]);
    engine.input(&input.date.year().to_le_bytes());
    engine.input(&[
        input.date.month(),
        input.date.day(),
        input.aggregation as u8,
        input.weight as u8,
    ]);
    engine.input(&u64::from(input.close).to_le_bytes());
    for (price, sats) in input.entries() {
        engine.input(&price.inner().to_le_bytes());
        engine.input(&u64::from(*sats).to_le_bytes());
    }
    Ok(sha256::Hash::from_engine(engine))
}
