//! BIP352's official send-and-receive test vectors (bitcoin/bips, `bip-0352/send_and_receive_test_vectors.json`):
//! every receiving case's inputs, each distinct case once, and the tweak it expects.

use std::str::FromStr;

use bitcoin::{
    OutPoint, ScriptBuf, TxIn, Txid, Witness, consensus::deserialize, hex::FromHex,
    secp256k1::PublicKey,
};
use brk_types::OutputType;

use crate::tweak::{Prevout, Tweaker, input_key, smallest_outpoint};

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
