//! BIP352's official send-and-receive test vectors (bitcoin/bips, `bip-0352/send_and_receive_test_vectors.json`):
//! every receiving case's inputs, each distinct case once, and the tweak it expects. Then the block
//! filters, against Bitcoin Core's BIP158 vectors, and the spent outputs' hashing.

use std::str::FromStr;

use bitcoin::{
    Amount, Block, BlockHash, OutPoint, ScriptBuf, Transaction, TxIn, TxOut, Txid, Witness,
    absolute::LockTime, consensus::deserialize, hashes::Hash, hex::FromHex,
    opcodes::all::OP_RETURN, secp256k1::PublicKey, transaction::Version,
};
use brk_types::{OutputType, Sats};

use crate::{
    compute::batches,
    filters::{filter, new_outputs, spent_outputs},
    scan::is_candidate,
    tweak::{Prevout, Tweaker, input_key, smallest_outpoint},
};

struct Input {
    txid: &'static str,
    vout: u32,
    script_sig: &'static str,
    witness: &'static str,
    prevout: &'static str,
}

struct Case {
    comment: &'static str,
    inputs: &'static [Input],
    tweak: Option<&'static str>,
}

fn bytes(hex: &str) -> Vec<u8> {
    Vec::from_hex(hex).unwrap()
}

/// The prevout as the scan sees it: the indexer's output type and address payload.
fn prevout(script: &ScriptBuf) -> Option<Prevout> {
    let bytes = script.as_bytes();
    match OutputType::from(script) {
        OutputType::P2PKH => Some(Prevout::P2PKH(bytes[3..23].try_into().unwrap())),
        OutputType::P2SH => Some(Prevout::P2SH),
        OutputType::P2WPKH => Some(Prevout::P2WPKH),
        OutputType::P2TR => Some(Prevout::P2TR(bytes[2..34].try_into().unwrap())),
        _ => None,
    }
}

#[test]
fn bip352_tweaks() {
    let tweaker = Tweaker::default();
    for case in CASES {
        let mut keys = Vec::<PublicKey>::new();
        let mut txins = Vec::new();
        for input in case.inputs {
            let witness = if input.witness.is_empty() {
                Witness::new()
            } else {
                deserialize::<Witness>(&bytes(input.witness)).unwrap()
            };
            let txin = TxIn {
                previous_output: OutPoint::new(Txid::from_str(input.txid).unwrap(), input.vout),
                script_sig: ScriptBuf::from_bytes(bytes(input.script_sig)),
                witness,
                ..TxIn::default()
            };
            let script = ScriptBuf::from_bytes(bytes(input.prevout));
            keys.extend(prevout(&script).and_then(|prevout| input_key(&prevout, &txin)));
            txins.push(txin);
        }
        let smallest = smallest_outpoint(&txins).unwrap();
        let tweak = tweaker.tweak(&smallest, &keys.iter().collect::<Vec<_>>());
        assert_eq!(
            tweak.map(|tweak| tweak.to_string()),
            case.tweak.map(str::to_owned),
            "{}",
            case.comment
        );
    }
}

/// The filters are BIP158's, byte for byte: Bitcoin Core's basic filter vectors, from each block's
/// output scripts (but empty and `OP_RETURN` ones) and the scripts its inputs spend.
#[test]
fn bip158_vectors() {
    for (hash, block, spent, expected) in BIP158_CASES {
        let block = deserialize::<Block>(&bytes(block)).unwrap();
        let hash = BlockHash::from_str(hash).unwrap().to_byte_array();
        let outputs = block
            .txdata
            .iter()
            .flat_map(|tx| &tx.output)
            .map(|output| output.script_pubkey.to_bytes())
            .filter(|script| script.first().is_some_and(|&op| op != OP_RETURN.to_u8()));
        let spent = spent.iter().map(|script| bytes(script)).filter(|script| !script.is_empty());
        assert_eq!(filter(&hash, outputs.chain(spent)), bytes(expected), "block {hash:?}");
    }
}

/// A transaction's output types and values, the types its inputs spend, and whether it's kept.
type Candidate = (&'static [OutputType], &'static [u64], &'static [OutputType], bool);

/// A transaction is kept for a Taproot output of 600 sats or more (its smaller Taproot outputs
/// kept beside it) and an input that may carry a key.
#[test]
fn candidates() {
    use OutputType::{P2PKH, P2SH, P2TR, P2WPKH, P2WSH};
    let sats = |values: &[u64]| values.iter().map(|&v| Sats::new(v)).collect::<Vec<_>>();
    let cases: [Candidate; 6] = [
        (&[P2TR], &[600], &[P2WPKH], true),
        (&[P2TR], &[599], &[P2WPKH], false),
        (&[P2TR, P2TR], &[330, 5_000], &[P2PKH], true),
        (&[P2WPKH], &[5_000], &[P2TR], false),
        (&[P2TR], &[5_000], &[P2WSH], false),
        (&[P2TR], &[5_000], &[P2WSH, P2SH], true),
    ];
    for (outputs, values, inputs, expected) in cases {
        let candidate = is_candidate(outputs, &sats(values), inputs);
        assert_eq!(candidate, expected, "{outputs:?} {values:?} {inputs:?}");
    }
}

/// Every block is scanned once: the blocks before Taproot's activation in one batch, then batches
/// ending on multiples of 1,000.
#[test]
fn batch_boundaries() {
    assert_eq!(
        batches(0..712_345).collect::<Vec<_>>(),
        [0..709_632, 709_632..710_000, 710_000..711_000, 711_000..712_000, 712_000..712_345]
    );
    let within = 709_700..709_800;
    assert_eq!(batches(within.clone()).collect::<Vec<_>>(), [within]);
    assert_eq!(batches(5..5).count(), 0);
}

/// The new outputs' filter holds each Taproot output's x-only key, read through the indexer's
/// output types aligned across the block's transactions, and nothing else.
#[test]
fn new_output_keys() {
    let output = |script: Vec<u8>| TxOut {
        value: Amount::from_sat(1_000),
        script_pubkey: ScriptBuf::from_bytes(script),
    };
    let taproot = |key: u8| [&[0x51, 0x20][..], &[key; 32]].concat();
    let transaction = |output| Transaction {
        version: Version::TWO,
        lock_time: LockTime::ZERO,
        input: Vec::new(),
        output,
    };
    let txdata = [
        transaction(vec![output(taproot(1)), output([&[0x00, 0x14][..], &[2; 20]].concat())]),
        transaction(vec![output(taproot(1)), output(taproot(3))]),
    ];
    let types = [OutputType::P2TR, OutputType::P2WPKH, OutputType::P2TR, OutputType::P2TR];
    let keys = new_outputs(&txdata, &types).collect::<Vec<_>>();
    assert_eq!(keys, [[1; 32].as_slice(), &[1; 32], &[3; 32]]);
    let block_hash = [9; 32];
    assert_eq!(filter(&block_hash, keys), filter(&block_hash, [[1; 32], [3; 32]]));
}

/// A spend of a Taproot output hashes as BlindBit's clients test for it, its block's inputs aligned
/// with the types of what they spend: block 800,000's first one, as Bitcoin Core lists the block
/// and silentpayments.dev hashes it.
#[test]
fn spent_output_hashes() {
    let block_hash =
        BlockHash::from_str("00000000000000000002a7c4c1e48d76c5a37902165a270156b7a8d72728a054")
            .unwrap()
            .to_byte_array();
    let spending = |txid: &str, vout| TxIn {
        previous_output: OutPoint::new(Txid::from_str(txid).unwrap(), vout),
        ..TxIn::default()
    };
    let transaction = |input| Transaction {
        version: Version::TWO,
        lock_time: LockTime::ZERO,
        input,
        output: Vec::new(),
    };
    let txdata = [
        transaction(vec![TxIn::default()]),
        transaction(vec![
            spending("c3e417e84950e955b523d02be6805955db547d1da6b49094d8357cc236bc5a50", 1),
            spending("c3e417e84950e955b523d02be6805955db547d1da6b49094d8357cc236bc5a50", 0),
        ]),
    ];
    let input_types = [OutputType::UnknownScript, OutputType::P2TR, OutputType::P2WPKH];
    let hashes = spent_outputs(&block_hash, &txdata, &input_types).collect::<Vec<_>>();
    assert_eq!(hashes, [bytes("6e5de5825186ae76").as_slice()]);
}

/// Bitcoin Core's BIP158 vectors (`src/test/data/blockfilters.json`): block hash, block, the
/// scripts its inputs spend, and its basic filter.
const BIP158_CASES: &[(&str, &str, &[&str], &str)] = &[
    (
        "000000000933ea01ad0ee984209779baaec3ced90fa3f408719526f8d77f4943",
        "0100000000000000000000000000000000000000000000000000000000000000000000003ba3edfd7a7b12b27ac72c3e67768f617fc81bc3888a51323a9fb8aa4b1e5e4adae5494dffff001d1aa4ae180101000000010000000000000000000000000000000000000000000000000000000000000000ffffffff4d04ffff001d0104455468652054696d65732030332f4a616e2f32303039204368616e63656c6c6f72206f6e206272696e6b206f66207365636f6e64206261696c6f757420666f722062616e6b73ffffffff0100f2052a01000000434104678afdb0fe5548271967f1a67130b7105cd6a828e03909a67962e0ea1f61deb649f6bc3f4cef38c4f35504e51ec112de5c384df7ba0b8d578a4c702b6bf11d5fac00000000",
        &[],
        "019dfca8",
    ),
    (
        "0000000018b07dca1b28b4b5a119f6d6e71698ce1ed96f143f54179ce177a19c",
        "02000000abfaf47274223ca2fea22797e44498240e482cb4c2f2baea088962f800000000604b5b52c32305b15d7542071d8b04e750a547500005d4010727694b6e72a776e55d0d51ffff001d211806480201000000010000000000000000000000000000000000000000000000000000000000000000ffffffff0d038bc0000102062f503253482fffffffff01a078072a01000000232102971dd6034ed0cf52450b608d196c07d6345184fcb14deb277a6b82d526a6163dac0000000001000000081cefd96060ecb1c4fbe675ad8a4f8bdc61d634c52b3a1c4116dee23749fe80ff000000009300493046022100866859c21f306538152e83f115bcfbf59ab4bb34887a88c03483a5dff9895f96022100a6dfd83caa609bf0516debc2bf65c3df91813a4842650a1858b3f61cfa8af249014730440220296d4b818bb037d0f83f9f7111665f49532dfdcbec1e6b784526e9ac4046eaa602204acf3a5cb2695e8404d80bf49ab04828bcbe6fc31d25a2844ced7a8d24afbdff01ffffffff1cefd96060ecb1c4fbe675ad8a4f8bdc61d634c52b3a1c4116dee23749fe80ff020000009400483045022100e87899175991aa008176cb553c6f2badbb5b741f328c9845fcab89f8b18cae2302200acce689896dc82933015e7230e5230d5cff8a1ffe82d334d60162ac2c5b0c9601493046022100994ad29d1e7b03e41731a4316e5f4992f0d9b6e2efc40a1ccd2c949b461175c502210099b69fdc2db00fbba214f16e286f6a49e2d8a0d5ffc6409d87796add475478d601ffffffff1e4a6d2d280ea06680d6cf8788ac90344a9c67cca9b06005bbd6d3f6945c8272010000009500493046022100a27400ba52fd842ce07398a1de102f710a10c5599545e6c95798934352c2e4df022100f6383b0b14c9f64b6718139f55b6b9494374755b86bae7d63f5d3e583b57255a01493046022100fdf543292f34e1eeb1703b264965339ec4a450ec47585009c606b3edbc5b617b022100a5fbb1c8de8aaaa582988cdb23622838e38de90bebcaab3928d949aa502a65d401ffffffff1e4a6d2d280ea06680d6cf8788ac90344a9c67cca9b06005bbd6d3f6945c8272020000009400493046022100ac626ac3051f875145b4fe4cfe089ea895aac73f65ab837b1ac30f5d875874fa022100bc03e79fa4b7eb707fb735b95ff6613ca33adeaf3a0607cdcead4cfd3b51729801483045022100b720b04a5c5e2f61b7df0fcf334ab6fea167b7aaede5695d3f7c6973496adbf1022043328c4cc1cdc3e5db7bb895ccc37133e960b2fd3ece98350f774596badb387201ffffffff23a8733e349c97d6cd90f520fdd084ba15ce0a395aad03cd51370602bb9e5db3010000004a00483045022100e8556b72c5e9c0da7371913a45861a61c5df434dfd962de7b23848e1a28c86ca02205d41ceda00136267281be0974be132ac4cda1459fe2090ce455619d8b91045e901ffffffff6856d609b881e875a5ee141c235e2a82f6b039f2b9babe82333677a5570285a6000000006a473044022040a1c631554b8b210fbdf2a73f191b2851afb51d5171fb53502a3a040a38d2c0022040d11cf6e7b41fe1b66c3d08f6ada1aee07a047cb77f242b8ecc63812c832c9a012102bcfad931b502761e452962a5976c79158a0f6d307ad31b739611dac6a297c256ffffffff6856d609b881e875a5ee141c235e2a82f6b039f2b9babe82333677a5570285a601000000930048304502205b109df098f7e932fbf71a45869c3f80323974a826ee2770789eae178a21bfc8022100c0e75615e53ee4b6e32b9bb5faa36ac539e9c05fa2ae6b6de5d09c08455c8b9601483045022009fb7d27375c47bea23b24818634df6a54ecf72d52e0c1268fb2a2c84f1885de022100e0ed4f15d62e7f537da0d0f1863498f9c7c0c0a4e00e4679588c8d1a9eb20bb801ffffffffa563c3722b7b39481836d5edfc1461f97335d5d1e9a23ade13680d0e2c1c371f030000006c493046022100ecc38ae2b1565643dc3c0dad5e961a5f0ea09cab28d024f92fa05c922924157e022100ebc166edf6fbe4004c72bfe8cf40130263f98ddff728c8e67b113dbd621906a601210211a4ed241174708c07206601b44a4c1c29e5ad8b1f731c50ca7e1d4b2a06dc1fffffffff02d0223a00000000001976a91445db0b779c0b9fa207f12a8218c94fc77aff504588ac80f0fa02000000000000000000",
        &["5221033423007d8f263819a2e42becaaf5b06f34cb09919e06304349d950668209eaed21021d69e2b68c3960903b702af7829fadcd80bd89b158150c85c4a75b2c8cb9c39452ae", "52210279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f8179821021d69e2b68c3960903b702af7829fadcd80bd89b158150c85c4a75b2c8cb9c39452ae", "522102a7ae1e0971fc1689bd66d2a7296da3a1662fd21a53c9e38979e0f090a375c12d21022adb62335f41eb4e27056ac37d462cda5ad783fa8e0e526ed79c752475db285d52ae", "52210279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f8179821022adb62335f41eb4e27056ac37d462cda5ad783fa8e0e526ed79c752475db285d52ae", "512103b9d1d0e2b4355ec3cdef7c11a5c0beff9e8b8d8372ab4b4e0aaf30e80173001951ae", "76a9149144761ebaccd5b4bbdc2a35453585b5637b2f8588ac", "522103f1848b40621c5d48471d9784c8174ca060555891ace6d2b03c58eece946b1a9121020ee5d32b54d429c152fdc7b1db84f2074b0564d35400d89d11870f9273ec140c52ae", "76a914f4fa1cc7de742d135ea82c17adf0bb9cf5f4fb8388ac"],
        "0afbc2920af1b027f31f87b592276eb4c32094bb4d3697021b4c6380",
    ),
    (
        "000000000000015d6077a411a8f5cc95caf775ccf11c54e27df75ce58d187313",
        "0000002060bbab0edbf3ef8a49608ee326f8fd75c473b7e3982095e2d100000000000000c30134f8c9b6d2470488d7a67a888f6fa12f8692e0c3411fbfb92f0f68f67eedae03ca57ef13021acc22dc4105010000000001010000000000000000000000000000000000000000000000000000000000000000ffffffff2f0315230e0004ae03ca57043e3d1e1d0c8796bf579aef0c0000000000122f4e696e6a61506f6f6c2f5345475749542fffffffff038427a112000000001976a914876fbb82ec05caa6af7a3b5e5a983aae6c6cc6d688ac0000000000000000266a24aa21a9ed5c748e121c0fe146d973a4ac26fa4a68b0549d46ee22d25f50a5e46fe1b377ee00000000000000002952534b424c4f434b3acd16772ad61a3c5f00287480b720f6035d5e54c9efc71be94bb5e3727f10909001200000000000000000000000000000000000000000000000000000000000000000000000000100000000010145310e878941a1b2bc2d33797ee4d89d95eaaf2e13488063a2aa9a74490f510a0100000023220020b6744de4f6ec63cc92f7c220cdefeeb1b1bed2b66c8e5706d80ec247d37e65a1ffffffff01002d3101000000001976a9143ebc40e411ed3c76f86711507ab952300890397288ac0400473044022001dd489a5d4e2fbd8a3ade27177f6b49296ba7695c40dbbe650ea83f106415fd02200b23a0602d8ff1bdf79dee118205fc7e9b40672bf31563e5741feb53fb86388501483045022100f88f040e90cc5dc6c6189d04718376ac19ed996bf9e4a3c29c3718d90ffd27180220761711f16c9e3a44f71aab55cbc0634907a1fa8bb635d971a9a01d368727bea10169522103b3623117e988b76aaabe3d63f56a4fc88b228a71e64c4cc551d1204822fe85cb2103dd823066e096f72ed617a41d3ca56717db335b1ea47a1b4c5c9dbdd0963acba621033d7c89bd9da29fa8d44db7906a9778b53121f72191184a9fee785c39180e4be153ae00000000010000000120925534261de4dcebb1ed5ab1b62bfe7a3ef968fb111dc2c910adfebc6e3bdf010000006b483045022100f50198f5ae66211a4f485190abe4dc7accdabe3bc214ebc9ea7069b97097d46e0220316a70a03014887086e335fc1b48358d46cd6bdc9af3b57c109c94af76fc915101210316cff587a01a2736d5e12e53551b18d73780b83c3bfb4fcf209c869b11b6415effffffff0220a10700000000001976a91450333046115eaa0ac9e0216565f945070e44573988ac2e7cd01a000000001976a914c01a7ca16b47be50cbdbc60724f701d52d75156688ac00000000010000000203a25f58630d7a1ea52550365fd2156683f56daf6ca73a4b4bbd097e66516322010000006a47304402204efc3d70e4ca3049c2a425025edf22d5ca355f9ec899dbfbbeeb2268533a0f2b02204780d3739653035af4814ea52e1396d021953f948c29754edd0ee537364603dc012103f7a897e4dbecab2264b21917f90664ea8256189ea725d28740cf7ba5d85b5763ffffffff03a25f58630d7a1ea52550365fd2156683f56daf6ca73a4b4bbd097e66516322000000006a47304402202d96defdc5b4af71d6ba28c9a6042c2d5ee7bc6de565d4db84ef517445626e03022022da80320e9e489c8f41b74833dfb6a54a4eb5087cdb46eb663eef0b25caa526012103f7a897e4dbecab2264b21917f90664ea8256189ea725d28740cf7ba5d85b5763ffffffff0200e1f5050000000017a914b7e6f7ff8658b2d1fb107e3d7be7af4742e6b1b3876f88fc00000000001976a914913bcc2be49cb534c20474c4dee1e9c4c317e7eb88ac0000000001000000043ffd60d3818431c495b89be84afac205d5d1ed663009291c560758bbd0a66df5010000006b483045022100f344607de9df42049688dcae8ff1db34c0c7cd25ec05516e30d2bc8f12ac9b2f022060b648f6a21745ea6d9782e17bcc4277b5808326488a1f40d41e125879723d3a012103f7a897e4dbecab2264b21917f90664ea8256189ea725d28740cf7ba5d85b5763ffffffffa5379401cce30f84731ef1ba65ce27edf2cc7ce57704507ebe8714aa16a96b92010000006a473044022020c37a63bf4d7f564c2192528709b6a38ab8271bd96898c6c2e335e5208661580220435c6f1ad4d9305d2c0a818b2feb5e45d443f2f162c0f61953a14d097fd07064012103f7a897e4dbecab2264b21917f90664ea8256189ea725d28740cf7ba5d85b5763ffffffff70e731e193235ff12c3184510895731a099112ffca4b00246c60003c40f843ce000000006a473044022053760f74c29a879e30a17b5f03a5bb057a5751a39f86fa6ecdedc36a1b7db04c022041d41c9b95f00d2d10a0373322a9025dba66c942196bc9d8adeb0e12d3024728012103f7a897e4dbecab2264b21917f90664ea8256189ea725d28740cf7ba5d85b5763ffffffff66b7a71b3e50379c8e85fc18fe3f1a408fc985f257036c34702ba205cef09f6f000000006a4730440220499bf9e2db3db6e930228d0661395f65431acae466634d098612fd80b08459ee022040e069fc9e3c60009f521cef54c38aadbd1251aee37940e6018aadb10f194d6a012103f7a897e4dbecab2264b21917f90664ea8256189ea725d28740cf7ba5d85b5763ffffffff0200e1f5050000000017a9148fc37ad460fdfbd2b44fe446f6e3071a4f64faa6878f447f0b000000001976a914913bcc2be49cb534c20474c4dee1e9c4c317e7eb88ac00000000",
        &["a914feb8a29635c56d9cd913122f90678756bf23887687", "76a914c01a7ca16b47be50cbdbc60724f701d52d75156688ac", "76a914913bcc2be49cb534c20474c4dee1e9c4c317e7eb88ac", "76a914913bcc2be49cb534c20474c4dee1e9c4c317e7eb88ac", "76a914913bcc2be49cb534c20474c4dee1e9c4c317e7eb88ac", "76a914913bcc2be49cb534c20474c4dee1e9c4c317e7eb88ac", "76a914913bcc2be49cb534c20474c4dee1e9c4c317e7eb88ac", "76a914913bcc2be49cb534c20474c4dee1e9c4c317e7eb88ac"],
        "09027acea61b6cc3fb33f5d52f7d088a6b2f75d234e89ca800",
    ),
    (
        "000000006f27ddfe1dd680044a34548f41bed47eba9e6f0b310da21423bc5f33",
        "000000201c8d1a529c39a396db2db234d5ec152fa651a2872966daccbde028b400000000083f14492679151dbfaa1a825ef4c18518e780c1f91044180280a7d33f4a98ff5f45765aaddc001d38333b9a02010000000001010000000000000000000000000000000000000000000000000000000000000000ffffffff230352471300fe5f45765afe94690a000963676d696e6572343208000000000000000000ffffffff024423a804000000001976a914f2c25ac3d59f3d674b1d1d0a25c27339aaac0ba688ac0000000000000000266a24aa21a9edcb26cb3052426b9ebb4d19c819ef87c19677bbf3a7c46ef0855bd1b2abe83491012000000000000000000000000000000000000000000000000000000000000000000000000002000000000101d20978463906ba4ff5e7192494b88dd5eb0de85d900ab253af909106faa22cc5010000000004000000014777ff000000000016001446c29eabe8208a33aa1023c741fa79aa92e881ff0347304402207d7ca96134f2bcfdd6b536536fdd39ad17793632016936f777ebb32c22943fda02206014d2fb8a6aa58279797f861042ba604ebd2f8f61e5bddbd9d3be5a245047b201004b632103eeaeba7ce5dc2470221e9517fb498e8d6bd4e73b85b8be655196972eb9ccd5566754b2752103a40b74d43df244799d041f32ce1ad515a6cd99501701540e38750d883ae21d3a68ac00000000",
        &["002027a5000c7917f785d8fc6e5a55adfca8717ecb973ebb7743849ff956d896a7ed"],
        "0385acb4f0fe889ef0",
    ),
    (
        "0000000000000027b2b3b3381f114f674f481544ff2be37ae3788d7e078383b1",
        "000000204ea88307a7959d8207968f152bedca5a93aefab253f1fb2cfb032a400000000070cebb14ec6dbc27a9dfd066d9849a4d3bac5f674665f73a5fe1de01a022a0c851fda85bf05f4c19a779d1450102000000010000000000000000000000000000000000000000000000000000000000000000ffffffff18034d94154d696e6572476174653030310d000000f238f401ffffffff01c817a804000000000000000000",
        &[],
        "00",
    ),
];

const CASES: &[Case] = &[
    Case {
        comment: "Simple send: two inputs",
        inputs: &[
            Input {
                txid: "f4184fc596403b9d638783cf57adfe4c75c605f6356fbc91338530e9831e9e16",
                vout: 0,
                script_sig: "493046022100ad79e6801dd9a8727f342f31c71c4912866f59dc6e7981878e92c5844a0ce929022100fb0d2393e813968648b9753b7e9871d90ab3d815ebf91820d704b19f4ed224d60121025a1e61f898173040e20616d43e9f496fba90338a39faa1ed98fcbaeee4dd9be5",
                witness: "",
                prevout: "76a91419c2f3ae0ca3b642bd3e49598b8da89f50c1416188ac",
            },
            Input {
                txid: "a1075db55d416d3ca199f55b6084e2115b9345e16c5cf302fc80e9d5fbf5d48d",
                vout: 0,
                script_sig: "49304602210086783ded73e961037e77d49d9deee4edc2b23136e9728d56e4491c80015c3a63022100fda4c0f21ea18de29edbce57f7134d613e044ee150a89e2e64700de2d4e83d4e012103bd85685d03d111699b15d046319febe77f8de5286e9e512703cdee1bf3be3792",
                witness: "",
                prevout: "76a914d9317c66f54ff0a152ec50b1d19c25be50c8e15988ac",
            },
        ],
        tweak: Some("024ac253c216532e961988e2a8ce266a447c894c781e52ef6cee902361db960004"),
    },
    Case {
        comment: "Simple send: two inputs, order reversed",
        inputs: &[
            Input {
                txid: "a1075db55d416d3ca199f55b6084e2115b9345e16c5cf302fc80e9d5fbf5d48d",
                vout: 0,
                script_sig: "493046022100ad79e6801dd9a8727f342f31c71c4912866f59dc6e7981878e92c5844a0ce929022100fb0d2393e813968648b9753b7e9871d90ab3d815ebf91820d704b19f4ed224d60121025a1e61f898173040e20616d43e9f496fba90338a39faa1ed98fcbaeee4dd9be5",
                witness: "",
                prevout: "76a91419c2f3ae0ca3b642bd3e49598b8da89f50c1416188ac",
            },
            Input {
                txid: "f4184fc596403b9d638783cf57adfe4c75c605f6356fbc91338530e9831e9e16",
                vout: 0,
                script_sig: "49304602210086783ded73e961037e77d49d9deee4edc2b23136e9728d56e4491c80015c3a63022100fda4c0f21ea18de29edbce57f7134d613e044ee150a89e2e64700de2d4e83d4e012103bd85685d03d111699b15d046319febe77f8de5286e9e512703cdee1bf3be3792",
                witness: "",
                prevout: "76a914d9317c66f54ff0a152ec50b1d19c25be50c8e15988ac",
            },
        ],
        tweak: Some("024ac253c216532e961988e2a8ce266a447c894c781e52ef6cee902361db960004"),
    },
    Case {
        comment: "Simple send: two inputs from the same transaction",
        inputs: &[
            Input {
                txid: "f4184fc596403b9d638783cf57adfe4c75c605f6356fbc91338530e9831e9e16",
                vout: 3,
                script_sig: "493046022100ad79e6801dd9a8727f342f31c71c4912866f59dc6e7981878e92c5844a0ce929022100fb0d2393e813968648b9753b7e9871d90ab3d815ebf91820d704b19f4ed224d60121025a1e61f898173040e20616d43e9f496fba90338a39faa1ed98fcbaeee4dd9be5",
                witness: "",
                prevout: "76a91419c2f3ae0ca3b642bd3e49598b8da89f50c1416188ac",
            },
            Input {
                txid: "f4184fc596403b9d638783cf57adfe4c75c605f6356fbc91338530e9831e9e16",
                vout: 7,
                script_sig: "49304602210086783ded73e961037e77d49d9deee4edc2b23136e9728d56e4491c80015c3a63022100fda4c0f21ea18de29edbce57f7134d613e044ee150a89e2e64700de2d4e83d4e012103bd85685d03d111699b15d046319febe77f8de5286e9e512703cdee1bf3be3792",
                witness: "",
                prevout: "76a914d9317c66f54ff0a152ec50b1d19c25be50c8e15988ac",
            },
        ],
        tweak: Some("03aeea547819c08413974e2ab2b12212e007166bb2058f88b009e082b9b4914a58"),
    },
    Case {
        comment: "Simple send: two inputs from the same transaction, order reversed",
        inputs: &[
            Input {
                txid: "a1075db55d416d3ca199f55b6084e2115b9345e16c5cf302fc80e9d5fbf5d48d",
                vout: 7,
                script_sig: "493046022100ad79e6801dd9a8727f342f31c71c4912866f59dc6e7981878e92c5844a0ce929022100fb0d2393e813968648b9753b7e9871d90ab3d815ebf91820d704b19f4ed224d60121025a1e61f898173040e20616d43e9f496fba90338a39faa1ed98fcbaeee4dd9be5",
                witness: "",
                prevout: "76a91419c2f3ae0ca3b642bd3e49598b8da89f50c1416188ac",
            },
            Input {
                txid: "a1075db55d416d3ca199f55b6084e2115b9345e16c5cf302fc80e9d5fbf5d48d",
                vout: 3,
                script_sig: "49304602210086783ded73e961037e77d49d9deee4edc2b23136e9728d56e4491c80015c3a63022100fda4c0f21ea18de29edbce57f7134d613e044ee150a89e2e64700de2d4e83d4e012103bd85685d03d111699b15d046319febe77f8de5286e9e512703cdee1bf3be3792",
                witness: "",
                prevout: "76a914d9317c66f54ff0a152ec50b1d19c25be50c8e15988ac",
            },
        ],
        tweak: Some("024cad5180a093d3af0f49f586bdf37f890920178e68e80561ed53351d0fa499ad"),
    },
    Case {
        comment: "Outpoint ordering byte-lexicographically vs. vout-integer",
        inputs: &[
            Input {
                txid: "f4184fc596403b9d638783cf57adfe4c75c605f6356fbc91338530e9831e9e16",
                vout: 1,
                script_sig: "493046022100ad79e6801dd9a8727f342f31c71c4912866f59dc6e7981878e92c5844a0ce929022100fb0d2393e813968648b9753b7e9871d90ab3d815ebf91820d704b19f4ed224d60121025a1e61f898173040e20616d43e9f496fba90338a39faa1ed98fcbaeee4dd9be5",
                witness: "",
                prevout: "76a91419c2f3ae0ca3b642bd3e49598b8da89f50c1416188ac",
            },
            Input {
                txid: "f4184fc596403b9d638783cf57adfe4c75c605f6356fbc91338530e9831e9e16",
                vout: 256,
                script_sig: "49304602210086783ded73e961037e77d49d9deee4edc2b23136e9728d56e4491c80015c3a63022100fda4c0f21ea18de29edbce57f7134d613e044ee150a89e2e64700de2d4e83d4e012103bd85685d03d111699b15d046319febe77f8de5286e9e512703cdee1bf3be3792",
                witness: "",
                prevout: "76a914d9317c66f54ff0a152ec50b1d19c25be50c8e15988ac",
            },
        ],
        tweak: Some("031f9a80d0938cf980b51f7cc4fad713d49037f430646dff129c0570d75a40d8f0"),
    },
    Case {
        comment: "Single recipient: multiple UTXOs from the same public key",
        inputs: &[
            Input {
                txid: "f4184fc596403b9d638783cf57adfe4c75c605f6356fbc91338530e9831e9e16",
                vout: 0,
                script_sig: "493046022100ad79e6801dd9a8727f342f31c71c4912866f59dc6e7981878e92c5844a0ce929022100fb0d2393e813968648b9753b7e9871d90ab3d815ebf91820d704b19f4ed224d60121025a1e61f898173040e20616d43e9f496fba90338a39faa1ed98fcbaeee4dd9be5",
                witness: "",
                prevout: "76a91419c2f3ae0ca3b642bd3e49598b8da89f50c1416188ac",
            },
            Input {
                txid: "a1075db55d416d3ca199f55b6084e2115b9345e16c5cf302fc80e9d5fbf5d48d",
                vout: 0,
                script_sig: "493046022100ad79e6801dd9a8727f342f31c71c4912866f59dc6e7981878e92c5844a0ce929022100fb0d2393e813968648b9753b7e9871d90ab3d815ebf91820d704b19f4ed224d60121025a1e61f898173040e20616d43e9f496fba90338a39faa1ed98fcbaeee4dd9be5",
                witness: "",
                prevout: "76a91419c2f3ae0ca3b642bd3e49598b8da89f50c1416188ac",
            },
        ],
        tweak: Some("0319949463fc6a2368d999a2a6a2bcb2dbf64a2ac6e00b3ba5659780c860a6d9e0"),
    },
    Case {
        comment: "Single recipient: taproot only inputs with even y-values",
        inputs: &[
            Input {
                txid: "f4184fc596403b9d638783cf57adfe4c75c605f6356fbc91338530e9831e9e16",
                vout: 0,
                script_sig: "",
                witness: "0140c459b671370d12cfb5acee76da7e3ba7cc29b0b4653e3af8388591082660137d087fdc8e89a612cd5d15be0febe61fc7cdcf3161a26e599a4514aa5c3e86f47b",
                prevout: "51205a1e61f898173040e20616d43e9f496fba90338a39faa1ed98fcbaeee4dd9be5",
            },
            Input {
                txid: "a1075db55d416d3ca199f55b6084e2115b9345e16c5cf302fc80e9d5fbf5d48d",
                vout: 0,
                script_sig: "",
                witness: "0140bd1e708f92dbeaf24a6b8dd22e59c6274355424d62baea976b449e220fd75b13578e262ab11b7aa58e037f0c6b0519b66803b7d9decaa1906dedebfb531c56c1",
                prevout: "5120782eeb913431ca6e9b8c2fd80a5f72ed2024ef72a3c6fb10263c379937323338",
            },
        ],
        tweak: Some("02dc59cc8e8873b65c1dd5c416d4fbeb647372c329bd84a70c05b310e222e2c183"),
    },
    Case {
        comment: "Single recipient: taproot only with mixed even/odd y-values",
        inputs: &[
            Input {
                txid: "f4184fc596403b9d638783cf57adfe4c75c605f6356fbc91338530e9831e9e16",
                vout: 0,
                script_sig: "",
                witness: "0140c459b671370d12cfb5acee76da7e3ba7cc29b0b4653e3af8388591082660137d087fdc8e89a612cd5d15be0febe61fc7cdcf3161a26e599a4514aa5c3e86f47b",
                prevout: "51205a1e61f898173040e20616d43e9f496fba90338a39faa1ed98fcbaeee4dd9be5",
            },
            Input {
                txid: "a1075db55d416d3ca199f55b6084e2115b9345e16c5cf302fc80e9d5fbf5d48d",
                vout: 0,
                script_sig: "",
                witness: "01400a4d0dca6293f40499394d7eefe14a1de11e0e3454f51de2e802592abf5ee549042a1b1a8fb2e149ee9dd3f086c1b69b2f182565ab6ecf599b1ec9ebadfda6c5",
                prevout: "51208c8d23d4764feffcd5e72e380802540fa0f88e3d62ad5e0b47955f74d7b283c4",
            },
        ],
        tweak: Some("03b990f5b1d90ea8fd4bdd5c856a9dfe17035d196958062e2c6cb4c99e413f3548"),
    },
    Case {
        comment: "Single recipient: taproot input with even y-value and non-taproot input",
        inputs: &[
            Input {
                txid: "f4184fc596403b9d638783cf57adfe4c75c605f6356fbc91338530e9831e9e16",
                vout: 0,
                script_sig: "",
                witness: "0140c459b671370d12cfb5acee76da7e3ba7cc29b0b4653e3af8388591082660137d087fdc8e89a612cd5d15be0febe61fc7cdcf3161a26e599a4514aa5c3e86f47b",
                prevout: "51205a1e61f898173040e20616d43e9f496fba90338a39faa1ed98fcbaeee4dd9be5",
            },
            Input {
                txid: "a1075db55d416d3ca199f55b6084e2115b9345e16c5cf302fc80e9d5fbf5d48d",
                vout: 0,
                script_sig: "473044021f24e010c6e475814740ba24c8cf9362c4db1276b7f46a7b1e63473159a80ec30221008198e8ece7b7f88e6c6cc6bb8c86f9f00b7458222a8c91addf6e1577bcf7697e012103e0ec4f64b3fa2e463ccfcf4e856e37d5e1e20275bc89ec1def9eb098eff1f85d",
                witness: "",
                prevout: "76a9148cbc7dfe44f1579bff3340bbef1eddeaeb1fc97788ac",
            },
        ],
        tweak: Some("0233c2a447b8b244e4ffcfb59fe365eaa3bb22288b31e2113b9998861f40d4d6da"),
    },
    Case {
        comment: "Single recipient: taproot input with odd y-value and non-taproot input",
        inputs: &[
            Input {
                txid: "f4184fc596403b9d638783cf57adfe4c75c605f6356fbc91338530e9831e9e16",
                vout: 0,
                script_sig: "",
                witness: "01400a4d0dca6293f40499394d7eefe14a1de11e0e3454f51de2e802592abf5ee549042a1b1a8fb2e149ee9dd3f086c1b69b2f182565ab6ecf599b1ec9ebadfda6c5",
                prevout: "51208c8d23d4764feffcd5e72e380802540fa0f88e3d62ad5e0b47955f74d7b283c4",
            },
            Input {
                txid: "a1075db55d416d3ca199f55b6084e2115b9345e16c5cf302fc80e9d5fbf5d48d",
                vout: 0,
                script_sig: "473044021f24e010c6e475814740ba24c8cf9362c4db1276b7f46a7b1e63473159a80ec30221008198e8ece7b7f88e6c6cc6bb8c86f9f00b7458222a8c91addf6e1577bcf7697e012103e0ec4f64b3fa2e463ccfcf4e856e37d5e1e20275bc89ec1def9eb098eff1f85d",
                witness: "",
                prevout: "76a9148cbc7dfe44f1579bff3340bbef1eddeaeb1fc97788ac",
            },
        ],
        tweak: Some("02d4e4f2c4cdb71c9c39a700a9ee1a0fc05b98362a441183f5770af7d6e2b3038c"),
    },
    Case {
        comment: "Multiple outputs: multiple outputs, same recipient",
        inputs: &[
            Input {
                txid: "f4184fc596403b9d638783cf57adfe4c75c605f6356fbc91338530e9831e9e16",
                vout: 0,
                script_sig: "493046022100ad79e6801dd9a8727f342f31c71c4912866f59dc6e7981878e92c5844a0ce929022100fb0d2393e813968648b9753b7e9871d90ab3d815ebf91820d704b19f4ed224d60121025a1e61f898173040e20616d43e9f496fba90338a39faa1ed98fcbaeee4dd9be5",
                witness: "",
                prevout: "76a91419c2f3ae0ca3b642bd3e49598b8da89f50c1416188ac",
            },
            Input {
                txid: "a1075db55d416d3ca199f55b6084e2115b9345e16c5cf302fc80e9d5fbf5d48d",
                vout: 0,
                script_sig: "483045022100a8c61b2d470e393279d1ba54f254b7c237de299580b7fa01ffcc940442ecec4502201afba952f4e4661c40acde7acc0341589031ba103a307b886eb867b23b850b97012103782eeb913431ca6e9b8c2fd80a5f72ed2024ef72a3c6fb10263c379937323338",
                witness: "",
                prevout: "76a9147cdd63cc408564188e8e472640e921c7c90e651d88ac",
            },
        ],
        tweak: Some("0314bec14463d6c0181083d607fecfba67bb83f95915f6f247975ec566d5642ee8"),
    },
    Case {
        comment: "Single recipient: taproot input with NUMS point",
        inputs: &[
            Input {
                txid: "f4184fc596403b9d638783cf57adfe4c75c605f6356fbc91338530e9831e9e16",
                vout: 0,
                script_sig: "",
                witness: "0440c459b671370d12cfb5acee76da7e3ba7cc29b0b4653e3af8388591082660137d087fdc8e89a612cd5d15be0febe61fc7cdcf3161a26e599a4514aa5c3e86f47b22205a1e61f898173040e20616d43e9f496fba90338a39faa1ed98fcbaeee4dd9be5ac21c150929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac00150",
                prevout: "5120da6f0595ecb302bbe73e2f221f05ab10f336b06817d36fd28fc6691725ddaa85",
            },
            Input {
                txid: "a1075db55d416d3ca199f55b6084e2115b9345e16c5cf302fc80e9d5fbf5d48d",
                vout: 0,
                script_sig: "",
                witness: "0140bd1e708f92dbeaf24a6b8dd22e59c6274355424d62baea976b449e220fd75b13578e262ab11b7aa58e037f0c6b0519b66803b7d9decaa1906dedebfb531c56c1",
                prevout: "5120782eeb913431ca6e9b8c2fd80a5f72ed2024ef72a3c6fb10263c379937323338",
            },
            Input {
                txid: "a1075db55d416d3ca199f55b6084e2115b9345e16c5cf302fc80e9d5fbf5d48d",
                vout: 1,
                script_sig: "",
                witness: "0340268d31a9276f6380107d5321cafa6d9e8e5ea39204318fdc8206b31507c891c3bbcea3c99e2208d73bd127a8e8c5f1e45a54f1bd217205414ddb566ab7eda0092220e0ec4f64b3fa2e463ccfcf4e856e37d5e1e20275bc89ec1def9eb098eff1f85dac21c050929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0",
                prevout: "51200a3c9365ceb131f89b0a4feb6896ebd67bb15a98c31eaa3da143bb955a0f3fcb",
            },
        ],
        tweak: Some("02213b872c9a6ee28a0d861384a1b3e3ec7257f4855ed09b4323e3899f3b028989"),
    },
    Case {
        comment: "Pubkey extraction from malleated p2pkh",
        inputs: &[
            Input {
                txid: "f4184fc596403b9d638783cf57adfe4c75c605f6356fbc91338530e9831e9e16",
                vout: 0,
                script_sig: "493046022100ad79e6801dd9a8727f342f31c71c4912866f59dc6e7981878e92c5844a0ce929022100fb0d2393e813968648b9753b7e9871d90ab3d815ebf91820d704b19f4ed224d60121025a1e61f898173040e20616d43e9f496fba90338a39faa1ed98fcbaeee4dd9be5",
                witness: "",
                prevout: "76a91419c2f3ae0ca3b642bd3e49598b8da89f50c1416188ac",
            },
            Input {
                txid: "f4184fc596403b9d638783cf57adfe4c75c605f6356fbc91338530e9831e9e16",
                vout: 1,
                script_sig: "0075483045022100a8c61b2d470e393279d1ba54f254b7c237de299580b7fa01ffcc940442ecec4502201afba952f4e4661c40acde7acc0341589031ba103a307b886eb867b23b850b97012103782eeb913431ca6e9b8c2fd80a5f72ed2024ef72a3c6fb10263c379937323338",
                witness: "",
                prevout: "76a9147cdd63cc408564188e8e472640e921c7c90e651d88ac",
            },
            Input {
                txid: "f4184fc596403b9d638783cf57adfe4c75c605f6356fbc91338530e9831e9e16",
                vout: 2,
                script_sig: "5163483045022100e7d26e77290b37128f5215ade25b9b908ce87cc9a4d498908b5bb8fd6daa1b8d022002568c3a8226f4f0436510283052bfb780b76f3fe4aa60c4c5eb118e43b18737012102e0ec4f64b3fa2e463ccfcf4e856e37d5e1e20275bc89ec1def9eb098eff1f85d67493046022100c0d3c851d3bd562ae93d56bcefd735ea57c027af46145a4d5e9cac113bfeb0c2022100ee5b2239af199fa9b7aa1d98da83a29d0a2cf1e4f29e2f37134ce386d51c544c012102ad0f26ddc7b3fcc340155963b3051b85289c1869612ecb290184ac952e2864ec68",
                witness: "",
                prevout: "76a914c82c5ec473cbc6c86e5ef410e36f9495adcf979988ac",
            },
        ],
        tweak: Some("028d6617f9bfe08604beb2188f4eebec923f5f8cc436fa6d14e4256e49bc32e7c8"),
    },
    Case {
        comment: "P2PKH and P2WPKH Uncompressed Keys are skipped",
        inputs: &[
            Input {
                txid: "a1075db55d416d3ca199f55b6084e2115b9345e16c5cf302fc80e9d5fbf5d48d",
                vout: 0,
                script_sig: "493046022100ad79e6801dd9a8727f342f31c71c4912866f59dc6e7981878e92c5844a0ce929022100fb0d2393e813968648b9753b7e9871d90ab3d815ebf91820d704b19f4ed224d60121025a1e61f898173040e20616d43e9f496fba90338a39faa1ed98fcbaeee4dd9be5",
                witness: "",
                prevout: "76a91419c2f3ae0ca3b642bd3e49598b8da89f50c1416188ac",
            },
            Input {
                txid: "f4184fc596403b9d638783cf57adfe4c75c605f6356fbc91338530e9831e9e16",
                vout: 0,
                script_sig: "483045022100a8c61b2d470e393279d1ba54f254b7c237de299580b7fa01ffcc940442ecec4502201afba952f4e4661c40acde7acc0341589031ba103a307b886eb867b23b850b97014104782eeb913431ca6e9b8c2fd80a5f72ed2024ef72a3c6fb10263c3799373233387c5343bf58e23269e903335b958a12182f9849297321e8d710e49a8727129cab",
                witness: "",
                prevout: "76a9144b92ac4ac6fe6212393894addda332f2e47a315688ac",
            },
            Input {
                txid: "a1075db55d416d3ca199f55b6084e2115b9345e16c5cf302fc80e9d5fbf5d48d",
                vout: 1,
                script_sig: "",
                witness: "02483045022100e7d26e77290b37128f5215ade25b9b908ce87cc9a4d498908b5bb8fd6daa1b8d022002568c3a8226f4f0436510283052bfb780b76f3fe4aa60c4c5eb118e43b18737014104e0ec4f64b3fa2e463ccfcf4e856e37d5e1e20275bc89ec1def9eb098eff1f85d6fe8190e189be57d0d5bcd17dbcbcd04c9b4a1c5f605b10d5c90abfcc0d12884",
                prevout: "00140423f731a07491364e8dce98b7c00bda63336950",
            },
        ],
        tweak: Some("02b04034f00da0678507d1345b7d56fecef825a1151f9dc7d8ca6946452a9e1f43"),
    },
    Case {
        comment: "Skip invalid P2SH inputs",
        inputs: &[
            Input {
                txid: "f4184fc596403b9d638783cf57adfe4c75c605f6356fbc91338530e9831e9e16",
                vout: 0,
                script_sig: "16001419c2f3ae0ca3b642bd3e49598b8da89f50c14161",
                witness: "02493046022100ad79e6801dd9a8727f342f31c71c4912866f59dc6e7981878e92c5844a0ce929022100fb0d2393e813968648b9753b7e9871d90ab3d815ebf91820d704b19f4ed224d60121025a1e61f898173040e20616d43e9f496fba90338a39faa1ed98fcbaeee4dd9be5",
                prevout: "a9148629db5007d5fcfbdbb466637af09daf9125969387",
            },
            Input {
                txid: "f4184fc596403b9d638783cf57adfe4c75c605f6356fbc91338530e9831e9e16",
                vout: 1,
                script_sig: "1600144b92ac4ac6fe6212393894addda332f2e47a3156",
                witness: "02483045022100a8c61b2d470e393279d1ba54f254b7c237de299580b7fa01ffcc940442ecec4502201afba952f4e4661c40acde7acc0341589031ba103a307b886eb867b23b850b97014104782eeb913431ca6e9b8c2fd80a5f72ed2024ef72a3c6fb10263c3799373233387c5343bf58e23269e903335b958a12182f9849297321e8d710e49a8727129cab",
                prevout: "a9146c9bf136fbb7305fd99d771a95127fcf87dedd0d87",
            },
            Input {
                txid: "f4184fc596403b9d638783cf57adfe4c75c605f6356fbc91338530e9831e9e16",
                vout: 2,
                script_sig: "00493046022100ad79e6801dd9a8727f342f31c71c4912866f59dc6e7981878e92c5844a0ce929022100fb0d2393e813968648b9753b7e9871d90ab3d815ebf91820d704b19f4ed224d601483045022100a8c61b2d470e393279d1ba54f254b7c237de299580b7fa01ffcc940442ecec4502201afba952f4e4661c40acde7acc0341589031ba103a307b886eb867b23b850b97014c695221025a1e61f898173040e20616d43e9f496fba90338a39faa1ed98fcbaeee4dd9be52103782eeb913431ca6e9b8c2fd80a5f72ed2024ef72a3c6fb10263c3799373233382102e0ec4f64b3fa2e463ccfcf4e856e37d5e1e20275bc89ec1def9eb098eff1f85d53ae",
                witness: "",
                prevout: "a9141044ddc6cea09e4ac40fbec2ba34ad62de6db25b87",
            },
        ],
        tweak: Some("02b04034f00da0678507d1345b7d56fecef825a1151f9dc7d8ca6946452a9e1f43"),
    },
    Case {
        comment: "Recipient ignores unrelated outputs",
        inputs: &[
            Input {
                txid: "f4184fc596403b9d638783cf57adfe4c75c605f6356fbc91338530e9831e9e16",
                vout: 0,
                script_sig: "",
                witness: "0140c459b671370d12cfb5acee76da7e3ba7cc29b0b4653e3af8388591082660137d087fdc8e89a612cd5d15be0febe61fc7cdcf3161a26e599a4514aa5c3e86f47b",
                prevout: "51205a1e61f898173040e20616d43e9f496fba90338a39faa1ed98fcbaeee4dd9be5",
            },
            Input {
                txid: "a1075db55d416d3ca199f55b6084e2115b9345e16c5cf302fc80e9d5fbf5d48d",
                vout: 0,
                script_sig: "483045022100a8c61b2d470e393279d1ba54f254b7c237de299580b7fa01ffcc940442ecec4502201afba952f4e4661c40acde7acc0341589031ba103a307b886eb867b23b850b97012103782eeb913431ca6e9b8c2fd80a5f72ed2024ef72a3c6fb10263c379937323338",
                witness: "",
                prevout: "76a9147cdd63cc408564188e8e472640e921c7c90e651d88ac",
            },
        ],
        tweak: Some("0314bec14463d6c0181083d607fecfba67bb83f95915f6f247975ec566d5642ee8"),
    },
    Case {
        comment: "No valid inputs, sender generates no outputs",
        inputs: &[
            Input {
                txid: "f4184fc596403b9d638783cf57adfe4c75c605f6356fbc91338530e9831e9e16",
                vout: 0,
                script_sig: "493046022100ad79e6801dd9a8727f342f31c71c4912866f59dc6e7981878e92c5844a0ce929022100fb0d2393e813968648b9753b7e9871d90ab3d815ebf91820d704b19f4ed224d60141045a1e61f898173040e20616d43e9f496fba90338a39faa1ed98fcbaeee4dd9be5c61836c9b1688ba431f7ea3039742251f62f0dca3da1bee58a47fa9b456c2d52",
                witness: "",
                prevout: "76a914460e8b41545d2dbe7e0671f0f573e2232814260a88ac",
            },
            Input {
                txid: "a1075db55d416d3ca199f55b6084e2115b9345e16c5cf302fc80e9d5fbf5d48d",
                vout: 0,
                script_sig: "483045022100a8c61b2d470e393279d1ba54f254b7c237de299580b7fa01ffcc940442ecec4502201afba952f4e4661c40acde7acc0341589031ba103a307b886eb867b23b850b97014104782eeb913431ca6e9b8c2fd80a5f72ed2024ef72a3c6fb10263c3799373233387c5343bf58e23269e903335b958a12182f9849297321e8d710e49a8727129cab",
                witness: "",
                prevout: "76a9144b92ac4ac6fe6212393894addda332f2e47a315688ac",
            },
        ],
        tweak: None,
    },
    Case {
        comment: "Input keys sum up to zero / point at infinity: sending fails, receiver skips tx",
        inputs: &[
            Input {
                txid: "3a286147b25e16ae80aff406f2673c6e565418c40f45c071245cdebc8a94174e",
                vout: 0,
                script_sig: "",
                witness: "024730440220085003179ce1a3a88ce0069aa6ea045e140761ab88c22a26ae2a8cfe983a6e4602204a8a39940f0735c8a4424270ac8da65240c261ab3fda9272f6d6efbf9cfea366012102557ef3e55b0a52489b4454c1169e06bdea43687a69c1f190eb50781644ab6975",
                prevout: "00149d9e24f9fab4e35bf1a6df4b46cb533296ac0792",
            },
            Input {
                txid: "3a286147b25e16ae80aff406f2673c6e565418c40f45c071245cdebc8a94174e",
                vout: 1,
                script_sig: "",
                witness: "0247304402204586a68e1d97dd3c6928e3622799859f8c3b20c3c670cf654cc905c9be29fdb7022043fbcde1689f3f4045e8816caf6163624bd19e62e4565bc99f95c533e599782c012103557ef3e55b0a52489b4454c1169e06bdea43687a69c1f190eb50781644ab6975",
                prevout: "00149860538b5575962776ed0814ae222c7d60c72d7b",
            },
        ],
        tweak: None,
    },
    Case {
        comment: "Input keys intermediate sum is zero but final sum is non-zero",
        inputs: &[
            Input {
                txid: "3a286147b25e16ae80aff406f2673c6e565418c40f45c071245cdebc8a94174e",
                vout: 0,
                script_sig: "",
                witness: "0247304402203e5537fa8c876b3475e7efe4f1474b0f48b7a6e4169179db5de9bb5b55ad1bd10220200e06f8f4d29dbc48bbcdf90df3278e798ce6cbf3c9fbf90427599fde147867012102557ef3e55b0a52489b4454c1169e06bdea43687a69c1f190eb50781644ab6975",
                prevout: "00149d9e24f9fab4e35bf1a6df4b46cb533296ac0792",
            },
            Input {
                txid: "3a286147b25e16ae80aff406f2673c6e565418c40f45c071245cdebc8a94174e",
                vout: 1,
                script_sig: "",
                witness: "0247304402207fdad0faf46edc54f5a5c67d33b2fa8d3f1fdc869381fd96e659f9e0c470ab1e022044f0d973339618b18667cef9a6251817f7f431f7f2b252a8cb760ccb40e7d823012103557ef3e55b0a52489b4454c1169e06bdea43687a69c1f190eb50781644ab6975",
                prevout: "00149860538b5575962776ed0814ae222c7d60c72d7b",
            },
            Input {
                txid: "3a286147b25e16ae80aff406f2673c6e565418c40f45c071245cdebc8a94174e",
                vout: 2,
                script_sig: "",
                witness: "0247304402203e5537fa8c876b3475e7efe4f1474b0f48b7a6e4169179db5de9bb5b55ad1bd10220200e06f8f4d29dbc48bbcdf90df3278e798ce6cbf3c9fbf90427599fde147867012102557ef3e55b0a52489b4454c1169e06bdea43687a69c1f190eb50781644ab6975",
                prevout: "00149d9e24f9fab4e35bf1a6df4b46cb533296ac0792",
            },
        ],
        tweak: Some("039c68bacb7efbf2175d781822f460afc4839a5798fabb055b50d939231bf57bb6"),
    },
    Case {
        comment: "Maximum per-group recipient limit K_max is exceeded (2324 matches): sending fails, receiver doesn't scan beyond limit",
        inputs: &[
            Input {
                txid: "3cde0e7ad8adf24a4e52fd18efafffb94f73f4c7facf2627a5b588ad1c9db87f",
                vout: 0,
                script_sig: "",
                witness: "01407ce01bc5ef2e4aae353e7f291fec32623d6a491b5421be6dccc70345f32ebae5baa6654c60c074e72da76fd06b343474370e56d3ec0413156ec6261757125ad4",
                prevout: "5120bca87f72e604e8850064552bedf380ca4584227057efe12a6cc238470658aaa3",
            },
        ],
        tweak: Some("022c333a8938ca0c7433c5e39d440b469f58374d0363b5bffc66aca1cc10d7b609"),
    },
    Case {
        comment: "Single recipient: taproot key-path spend with annex",
        inputs: &[
            Input {
                txid: "f4184fc596403b9d638783cf57adfe4c75c605f6356fbc91338530e9831e9e16",
                vout: 0,
                script_sig: "",
                witness: "0240c459b671370d12cfb5acee76da7e3ba7cc29b0b4653e3af8388591082660137d087fdc8e89a612cd5d15be0febe61fc7cdcf3161a26e599a4514aa5c3e86f47b215050929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0",
                prevout: "51205a1e61f898173040e20616d43e9f496fba90338a39faa1ed98fcbaeee4dd9be5",
            },
            Input {
                txid: "a1075db55d416d3ca199f55b6084e2115b9345e16c5cf302fc80e9d5fbf5d48d",
                vout: 0,
                script_sig: "",
                witness: "0140bd1e708f92dbeaf24a6b8dd22e59c6274355424d62baea976b449e220fd75b13578e262ab11b7aa58e037f0c6b0519b66803b7d9decaa1906dedebfb531c56c1",
                prevout: "5120782eeb913431ca6e9b8c2fd80a5f72ed2024ef72a3c6fb10263c379937323338",
            },
        ],
        tweak: Some("02dc59cc8e8873b65c1dd5c416d4fbeb647372c329bd84a70c05b310e222e2c183"),
    },
];
