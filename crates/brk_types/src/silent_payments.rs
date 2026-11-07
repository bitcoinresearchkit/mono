crate::bytes::hex_bytes! {
    /// A BIP352 silent payments tweak, `input_hash·A`: the compressed point a wallet multiplies by
    /// its scan key for the secret it shares with the transaction.
    Tweak(33),
    /// A tweak's x coordinate, kept apart from its parity byte (a [`Tweak`]'s first), which then
    /// compresses to a bit.
    TweakX(32),
    /// A spend of a Taproot output as BlindBit's light clients test for it: the first 8 bytes of
    /// `sha256(txid ‖ vout ‖ block hash)`, the txid and the spending block's hash in internal
    /// byte order, the vout little-endian.
    SpentOutputHash(8),
}
