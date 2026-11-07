crate::bytes::hex_bytes! {
    /// A BIP352 silent payments tweak, `input_hash·A`: the compressed point a wallet multiplies by
    /// its scan key for the secret it shares with the transaction.
    Tweak(33),
    /// A tweak's x coordinate, kept apart from its parity byte (a [`Tweak`]'s first), which then
    /// compresses to a bit.
    TweakX(32),
    /// The first 4 bytes of a Taproot output key: a silent payments wallet matches the key it
    /// computes against them, a false match about once in 4 billion outputs.
    TaprootPrefix(4),
}
