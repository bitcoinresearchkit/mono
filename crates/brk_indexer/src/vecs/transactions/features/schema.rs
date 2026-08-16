macro_rules! with_transaction_features {
    ($macro:ident) => {
        $macro! {
            /// Whether the transaction creates or, outside coinbase, spends at
            /// least one P2PK-shaped output with a 33- or 65-byte key field.
            has_p2pk: P2PK = 0, count: p2pk, count_attr: doc = "Number of transactions in the block that create or, outside coinbase, spend at least one P2PK-shaped output with a 33- or 65-byte key field.";
            /// Whether the transaction creates or, outside coinbase, spends at
            /// least one bare multisig output recognized by Bitcoin script
            /// parsing.
            has_p2ms: P2MS = 1, count: p2ms, count_attr: doc = "Number of transactions in the block that create or, outside coinbase, spend at least one bare multisig output recognized by Bitcoin script parsing.";
            /// Whether the transaction creates or, outside coinbase, spends at
            /// least one pay-to-public-key-hash output.
            has_p2pkh: P2PKH = 2, count: p2pkh, count_attr: doc = "Number of transactions in the block that create or, outside coinbase, spend at least one pay-to-public-key-hash output.";
            /// Whether the transaction creates or, outside coinbase, spends at
            /// least one pay-to-script-hash output.
            has_p2sh: P2SH = 3, count: p2sh, count_attr: doc = "Number of transactions in the block that create or, outside coinbase, spend at least one pay-to-script-hash output.";
            /// Whether the transaction creates or, outside coinbase, spends at
            /// least one version-0 pay-to-witness-public-key-hash output.
            has_p2wpkh: P2WPKH = 4, count: p2wpkh, count_attr: doc = "Number of transactions in the block that create or, outside coinbase, spend at least one version-0 pay-to-witness-public-key-hash output.";
            /// Whether the transaction creates or, outside coinbase, spends at
            /// least one version-0 pay-to-witness-script-hash output.
            has_p2wsh: P2WSH = 5, count: p2wsh, count_attr: doc = "Number of transactions in the block that create or, outside coinbase, spend at least one version-0 pay-to-witness-script-hash output.";
            /// Whether the transaction creates or, outside coinbase, spends at
            /// least one pay-to-Taproot output.
            has_p2tr: P2TR = 6, count: p2tr, count_attr: doc = "Number of transactions in the block that create or, outside coinbase, spend at least one pay-to-Taproot output.";
            /// Whether the transaction creates or, outside coinbase, spends at
            /// least one pay-to-Anchor output matching
            /// `OP_1 PUSHBYTES_2 0x4e73`.
            has_p2a: P2A = 7, count: p2a, count_attr: doc = "Number of transactions in the block that create or, outside coinbase, spend at least one pay-to-Anchor output matching `OP_1 PUSHBYTES_2 0x4e73`.";
            /// Whether the transaction creates or, outside coinbase, spends at
            /// least one output whose locking script begins with `OP_RETURN`.
            has_op_return: OP_RETURN = 8, count: op_return, count_attr: doc = "Number of transactions in the block that create or, outside coinbase, spend at least one output whose locking script begins with `OP_RETURN`.";
            /// Whether the transaction creates or, outside coinbase, spends at
            /// least one output with an empty locking script.
            has_empty: EMPTY = 9, count: empty, count_attr: doc = "Number of transactions in the block that create or, outside coinbase, spend at least one output with an empty locking script.";
            /// Whether the transaction creates or, outside coinbase, spends at
            /// least one output not matching another recognized locking-script
            /// type.
            has_unknown: UNKNOWN = 10, count: unknown, count_attr: doc = "Number of transactions in the block that create or, outside coinbase, spend at least one output not matching another recognized locking-script type.";
            /// Whether the transaction creates a P2PK output whose shaped
            /// public key is invalid, or a bare multisig output containing an
            /// invalid or recognized burn public key.
            has_fake_pubkey: FAKE_PUBKEY = 11, count: fake_pubkey, count_attr: doc = "Number of transactions in the block that create a P2PK output whose shaped public key is invalid, or a bare multisig output containing an invalid or recognized burn public key.";
            /// Whether the transaction contains a consecutive run of P2WSH
            /// outputs whose 32-byte programs encode a big-endian two-byte
            /// payload length and the required zero padding in the final
            /// program.
            has_fake_scripthash: FAKE_SCRIPTHASH = 12, count: fake_scripthash, count_attr: doc = "Number of transactions in the block containing a consecutive run of P2WSH outputs whose 32-byte programs encode a big-endian two-byte payload length and the required zero padding in the final program.";
            /// Whether at least one Taproot script-path input contains the
            /// Ordinals envelope prefix `OP_0 OP_IF PUSH 'ord'` in its
            /// tapscript.
            has_inscription: INSCRIPTION = 13, count: inscription, count_attr: traversable(hidden);
            /// Whether at least one Taproot input with more than one witness
            /// element ends in an annex whose first byte is `0x50`.
            has_annex: ANNEX = 14, count: annex, count_attr: traversable(hidden);
            /// Whether the transaction contains at least one detected ECDSA or
            /// Schnorr signature encoding using `SIGHASH_ALL`.
            has_sighash_all: SIGHASH_ALL = 15, count: sighash_all, count_attr: traversable(hidden);
            /// Whether the transaction contains at least one detected ECDSA or
            /// Schnorr signature encoding using `SIGHASH_NONE`.
            has_sighash_none: SIGHASH_NONE = 16, count: sighash_none, count_attr: traversable(hidden);
            /// Whether the transaction contains at least one detected ECDSA or
            /// Schnorr signature encoding using `SIGHASH_SINGLE`.
            has_sighash_single: SIGHASH_SINGLE = 17, count: sighash_single, count_attr: traversable(hidden);
            /// Whether the transaction contains at least one detected Taproot
            /// signature encoding using `SIGHASH_DEFAULT`.
            has_sighash_default: SIGHASH_DEFAULT = 18, count: sighash_default, count_attr: traversable(hidden);
            /// Whether the transaction contains at least one detected ECDSA or
            /// Schnorr signature encoding with the `SIGHASH_ANYONECANPAY`
            /// modifier. This is independent of ALL, NONE, and SINGLE.
            has_sighash_anyone_can_pay: SIGHASH_ANYONE_CAN_PAY = 19, count: sighash_anyone_can_pay, count_attr: traversable(hidden);
            #[traversable(hidden)]
            is_unconditionally_nonstandard: UNCONDITIONALLY_NONSTANDARD = 20;
            /// Whether a non-coinbase transaction has at least one output below
            /// BRK's type-specific dust threshold: 672 sats for P2PK65, 576 for
            /// P2PK33, 546 for P2PKH, 540 for P2SH, 294 for P2WPKH, 330 for
            /// P2WSH or P2TR, 240 for P2A, and 471 for an empty script. P2MS and
            /// unknown scripts use their computed minimal non-dust value;
            /// OP_RETURN is excluded.
            has_dust_output: DUST_OUTPUT = 21, count: dust_output, count_attr: traversable(hidden);
        }
    };
}

pub(super) use with_transaction_features;
