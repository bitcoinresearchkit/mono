crate::bytes::hex_bytes! {
    /// A BIP352 silent payments tweak, `input_hash·A`: the compressed point a wallet multiplies by
    /// its scan key for the secret it shares with the transaction.
    Tweak(33),
}
