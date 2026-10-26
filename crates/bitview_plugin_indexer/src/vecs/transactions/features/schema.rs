macro_rules! with_transaction_features {
    ($macro:ident) => {
        $macro! {
            features {
                /// Transactions that create or, outside coinbase, spend at least one P2PK-shaped output with a 33- or
                /// 65-byte key field.
                #[traversable(lead = "Transactions with pay-to-public-key (P2PK) outputs, made or spent.")]
                p2pk: has_p2pk, P2PK = 0;
                /// Transactions that create or, outside coinbase, spend at least one pay-to-public-key-hash output.
                #[traversable(lead = "Transactions with pay-to-public-key-hash (P2PKH) outputs, made or spent.")]
                p2pkh: has_p2pkh, P2PKH = 2;
                /// Transactions that create or, outside coinbase, spend at least one bare multisig output recognized
                /// by Bitcoin script parsing.
                #[traversable(lead = "Transactions with bare multisig (P2MS) outputs, made or spent.")]
                p2ms: has_p2ms, P2MS = 1;
                /// Transactions that create or, outside coinbase, spend at least one pay-to-script-hash output.
                #[traversable(lead = "Transactions with pay-to-script-hash (P2SH) outputs, made or spent.")]
                p2sh: has_p2sh, P2SH = 3;
                /// Transactions that create or, outside coinbase, spend at least one version-0
                /// pay-to-witness-public-key-hash output.
                #[traversable(lead = "Transactions with SegWit P2WPKH outputs, made or spent.")]
                p2wpkh: has_p2wpkh, P2WPKH = 4;
                /// Transactions that create or, outside coinbase, spend at least one version-0
                /// pay-to-witness-script-hash output.
                #[traversable(lead = "Transactions with SegWit P2WSH outputs, made or spent.")]
                p2wsh: has_p2wsh, P2WSH = 5;
                /// Transactions that create or, outside coinbase, spend at least one pay-to-Taproot output.
                #[traversable(lead = "Transactions with Taproot (P2TR) outputs, made or spent.")]
                p2tr: has_p2tr, P2TR = 6;
                /// Transactions that create or, outside coinbase, spend at least one pay-to-Anchor output matching
                /// `OP_1 PUSHBYTES_2 0x4e73`.
                #[traversable(lead = "Transactions with pay-to-anchor (P2A) outputs, made or spent.")]
                p2a: has_p2a, P2A = 7;
                /// Transactions that create or, outside coinbase, spend at least one output with an empty locking
                /// script.
                #[traversable(lead = "Transactions with empty-script outputs, made or spent.")]
                empty_script: has_empty_script, EMPTY = 9;
                /// Transactions that create or, outside coinbase, spend at least one output not matching another
                /// recognized locking-script type.
                #[traversable(lead = "Transactions with unrecognized outputs, made or spent.")]
                unknown_script: has_unknown_script, UNKNOWN = 10;
                /// Transactions that create at least one output whose locking script begins with `OP_RETURN`.
                #[traversable(lead = "Transactions with OP_RETURN outputs: data written on the chain.")]
                op_return: has_op_return, OP_RETURN = 8;
                /// Transactions that create a P2PK output whose shaped public key is invalid, or a bare multisig
                /// output containing an invalid or recognized burn public key.
                #[traversable(lead = "Transactions with invalid or burn public keys, often data written as keys.")]
                fake_pubkey: has_fake_pubkey, FAKE_PUBKEY = 11;
                /// Transactions containing a consecutive run of P2WSH outputs whose 32-byte programs encode a
                /// big-endian two-byte payload length and the required zero padding in the final program.
                #[traversable(lead = "Transactions with fake script hashes, encoding data.")]
                fake_scripthash: has_fake_scripthash, FAKE_SCRIPTHASH = 12;
                /// Transactions where at least one Taproot script-path input contains the Ordinals envelope prefix
                /// `OP_0 OP_IF PUSH 'ord'` in its tapscript.
                #[traversable(lead = "Inscriptions: transactions carrying Ordinals inscriptions.")]
                inscription: has_inscription, INSCRIPTION = 13;
                /// Transactions where at least one Taproot input with more than one witness element ends in an annex
                /// whose first byte is `0x50`.
                #[traversable(lead = "Transactions with a Taproot annex: an extra witness element, kept for future use.")]
                annex: has_annex, ANNEX = 14;
                /// Transactions containing at least one detected ECDSA or Schnorr signature using the `SIGHASH_ALL`
                /// base type, which commits to every output. `SIGHASH_ANYONECANPAY`, tracked separately, can narrow
                /// the input commitment.
                #[traversable(lead = "Transactions with SIGHASH_ALL signatures, the usual kind: committing to every output.")]
                sighash_all: has_sighash_all, SIGHASH_ALL = 15;
                /// Transactions containing at least one detected ECDSA or Schnorr signature using the `SIGHASH_NONE`
                /// base type, which commits to no transaction outputs.
                #[traversable(lead = "Transactions with SIGHASH_NONE signatures: committing to none of the outputs.")]
                sighash_none: has_sighash_none, SIGHASH_NONE = 16;
                /// Transactions containing at least one detected ECDSA or Schnorr signature using the
                /// `SIGHASH_SINGLE` base type, which normally commits only to the output at the signing input's
                /// position. Legacy signatures without a corresponding output retain Bitcoin's historical
                /// `SIGHASH_SINGLE` bug.
                #[traversable(lead = "Transactions with SIGHASH_SINGLE signatures: committing, normally, to one output.")]
                sighash_single: has_sighash_single, SIGHASH_SINGLE = 17;
                /// Transactions containing at least one detected Taproot `SIGHASH_DEFAULT` signature. Taproot's
                /// omitted hash-type byte has the same commitments as `SIGHASH_ALL` without `SIGHASH_ANYONECANPAY`.
                #[traversable(lead = "Transactions with Taproot's default signatures: committing to every input and output.")]
                sighash_default: has_sighash_default, SIGHASH_DEFAULT = 18;
                /// Transactions containing at least one detected ECDSA or Schnorr signature with the
                /// `SIGHASH_ANYONECANPAY` modifier, which commits only to the signing input rather than every input.
                /// This is independent of ALL, NONE, and SINGLE.
                #[traversable(lead = "Transactions with ANYONECANPAY signatures: committing to their own input alone, so others can add theirs.")]
                sighash_anyone_can_pay: has_sighash_anyone_can_pay, SIGHASH_ANYONE_CAN_PAY = 19;
                /// Transactions with at least one input sequence number below `0xfffffffe`, the explicit opt-in RBF
                /// signal defined by BIP 125. This is a mechanical sequence signal: it does not prove the transaction
                /// was replaceable or replaced, does not include inherited signaling, and does not account for
                /// full-RBF policy. Coinbase transactions are evaluated by the same sequence rule.
                #[traversable(lead = "Replace-by-fee transactions: ones that say they may be replaced by one paying more.")]
                rbf_signaling: is_rbf_signaling, RBF_SIGNALING = 22;
                /// Non-coinbase transactions with at least one output below the type-specific dust threshold: 672
                /// sats for P2PK65, 576 for P2PK33, 546 for P2PKH, 540 for P2SH, 294 for P2WPKH, 330 for P2WSH or
                /// P2TR, 240 for P2A, and 471 for an empty script. P2MS and unknown scripts use their computed
                /// minimal non-dust value; OP_RETURN is excluded.
                #[traversable(lead = "Transactions with dust: outputs below Bitcoin Core's dust threshold.")]
                dust_output: has_dust_output, DUST_OUTPUT = 21;
                /// Version-1 transactions tagged as EPOBC colored-coin transfers or issuances: the first
                /// input's sequence number is below 4,096 (bits 12 to 31 zero) and its low six bits are
                /// 51 (transfer) or 37 (genesis), the remaining bits giving the padding, as ChromaWay's
                /// reference implementation writes them, and the first input is not a Taproot spend (no EPOBC
                /// wallet made one; small sequence numbers there carry the tag by chance). EPOBC carries no
                /// `OP_RETURN` payload.
                #[traversable(lead = "EPOBC transactions: colored coins, an early way of issuing assets on Bitcoin.")]
                epobc: is_epobc, EPOBC = 23;
            }
            flags {
                #[traversable(hidden)]
                is_unconditionally_nonstandard: UNCONDITIONALLY_NONSTANDARD = 20;
            }
        }
    };
}
