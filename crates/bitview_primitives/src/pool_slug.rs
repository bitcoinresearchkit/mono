use std::{fmt, mem};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[cfg(feature = "storage")]
use vecdb::{Bytes, Error, Formattable, Result};

/// Declares `PoolSlug` with one spelling per variant, shared by serde (API paths and JSON),
/// `Display` (series ids) and `AsRef<str>`.
macro_rules! pool_slugs {
    ($($(#[$attr:meta])* $variant:ident => $slug:literal,)*) => {
        /// URL-friendly mining pool identifier
        #[allow(clippy::upper_case_acronyms)]
        #[derive(
            Default,
            Debug,
            Copy,
            Clone,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Hash,
            Serialize,
            Deserialize,
            JsonSchema,
        )]
        #[repr(u8)]
        pub enum PoolSlug {
            $($(#[$attr])* #[serde(rename = $slug)] $variant,)*
        }

        impl PoolSlug {
            pub const fn as_str(self) -> &'static str {
                match self {
                    $(Self::$variant => $slug,)*
                }
            }
        }
    };
}

pool_slugs! {
    #[default] Unknown => "unknown",
    BlockFills => "blockfills",
    UltimusPool => "ultimuspool",
    TerraPool => "terrapool",
    Luxor => "luxor",
    OneThash => "1thash",
    BtcCom => "btccom",
    Bitfarms => "bitfarms",
    HuobiPool => "huobipool",
    WayiCn => "wayicn",
    CanoePool => "canoepool",
    BtcTop => "btctop",
    BitcoinCom => "bitcoincom",
    Pool175btc => "175btc",
    GbMiners => "gbminers",
    AXbt => "axbt",
    AsicMiner => "asicminer",
    BitMinter => "bitminter",
    BitcoinRussia => "bitcoinrussia",
    BtcServ => "btcserv",
    SimplecoinUs => "simplecoinus",
    BtcGuild => "btcguild",
    Eligius => "eligius",
    OzCoin => "ozcoin",
    EclipseMc => "eclipsemc",
    MaxBtc => "maxbtc",
    TripleMining => "triplemining",
    CoinLab => "coinlab",
    Pool50btc => "50btc",
    GhashIo => "ghashio",
    StMiningCorp => "stminingcorp",
    Bitparking => "bitparking",
    Mmpool => "mmpool",
    Polmine => "polmine",
    KncMiner => "kncminer",
    Bitalo => "bitalo",
    F2Pool => "f2pool",
    Hhtt => "hhtt",
    MegaBigPower => "megabigpower",
    MtRed => "mtred",
    NmcBit => "nmcbit",
    YourbtcNet => "yourbtcnet",
    GiveMeCoins => "givemecoins",
    BraiinsPool => "braiinspool",
    AntPool => "antpool",
    MultiCoinCo => "multicoinco",
    BcpoolIo => "bcpoolio",
    Cointerra => "cointerra",
    KanoPool => "kanopool",
    SoloCk => "solock",
    CkPool => "ckpool",
    NiceHash => "nicehash",
    BitClub => "bitclub",
    BitcoinAffiliateNetwork => "bitcoinaffiliatenetwork",
    Btcc => "btcc",
    BwPool => "bwpool",
    ExxBw => "exxbw",
    Bitsolo => "bitsolo",
    BitFury => "bitfury",
    TwentyOneInc => "21inc",
    DigitalBtc => "digitalbtc",
    EightBaochi => "8baochi",
    MyBtcCoinPool => "mybtccoinpool",
    TbDice => "tbdice",
    HashPool => "hashpool",
    Nexious => "nexious",
    BravoMining => "bravomining",
    HotPool => "hotpool",
    OkExPool => "okexpool",
    BcMonster => "bcmonster",
    OneHash => "1hash",
    Bixin => "bixin",
    TatmasPool => "tatmaspool",
    ViaBtc => "viabtc",
    ConnectBtc => "connectbtc",
    BatPool => "batpool",
    Waterhole => "waterhole",
    DcExploration => "dcexploration",
    Dcex => "dcex",
    BtPool => "btpool",
    FiftyEightCoin => "58coin",
    BitcoinIndia => "bitcoinindia",
    ShawnP0wers => "shawnp0wers",
    PHashIo => "phashio",
    RigPool => "rigpool",
    HaoZhuZhu => "haozhuzhu",
    SevenPool => "7pool",
    MiningKings => "miningkings",
    HashBx => "hashbx",
    DPool => "dpool",
    Rawpool => "rawpool",
    Haominer => "haominer",
    Helix => "helix",
    BitcoinUkraine => "bitcoinukraine",
    Poolin => "poolin",
    SecretSuperstar => "secretsuperstar",
    TigerpoolNet => "tigerpoolnet",
    SigmapoolCom => "sigmapoolcom",
    OkpoolTop => "okpooltop",
    Hummerpool => "hummerpool",
    Tangpool => "tangpool",
    BytePool => "bytepool",
    SpiderPool => "spiderpool",
    NovaBlock => "novablock",
    MiningCity => "miningcity",
    BinancePool => "binancepool",
    Minerium => "minerium",
    LubianCom => "lubiancom",
    Okkong => "okkong",
    AaoPool => "aaopool",
    EmcdPool => "emcdpool",
    FoundryUsa => "foundryusa",
    SbiCrypto => "sbicrypto",
    ArkPool => "arkpool",
    PureBtcCom => "purebtccom",
    MaraPool => "marapool",
    KuCoinPool => "kucoinpool",
    EntrustCharityPool => "entrustcharitypool",
    OkMiner => "okminer",
    Titan => "titan",
    PegaPool => "pegapool",
    BtcNuggets => "btcnuggets",
    CloudHashing => "cloudhashing",
    DigitalXMintsy => "digitalxmintsy",
    Telco214 => "telco214",
    BtcPoolParty => "btcpoolparty",
    Multipool => "multipool",
    TransactionCoinMining => "transactioncoinmining",
    BtcDig => "btcdig",
    TrickysBtcPool => "trickysbtcpool",
    BtcMp => "btcmp",
    Eobot => "eobot",
    Unomp => "unomp",
    Patels => "patels",
    GoGreenLight => "gogreenlight",
    BitcoinIndiaPool => "bitcoinindiapool",
    EkanemBtc => "ekanembtc",
    Canoe => "canoe",
    Tiger => "tiger",
    OneM1x => "1m1x",
    Zulupool => "zulupool",
    SecPool => "secpool",
    Ocean => "ocean",
    WhitePool => "whitepool",
    Wiz => "wiz",
    #[serde(skip)] Dummy145 => "dummy145",
    #[serde(skip)] Dummy146 => "dummy146",
    Wk057 => "wk057",
    FutureBitApolloSolo => "futurebitapollosolo",
    #[serde(skip)] Dummy149 => "dummy149",
    #[serde(skip)] Dummy150 => "dummy150",
    CarbonNegative => "carbonnegative",
    PortlandHodl => "portlandhodl",
    Phoenix => "phoenix",
    Neopool => "neopool",
    MaxiPool => "maxipool",
    #[serde(skip)] Dummy156 => "dummy156",
    BitFuFuPool => "bitfufupool",
    GDPool => "gdpool",
    MiningDutch => "miningdutch",
    PublicPool => "publicpool",
    MiningSquared => "miningsquared",
    InnopolisTech => "innopolistech",
    #[serde(skip)] Dummy163 => "dummy163",
    BtcLab => "btclab",
    Parasite => "parasite",
    RedRockPool => "redrockpool",
    Est3lar => "est3lar",
    BraiinsSolo => "braiinssolo",
    SoloPool => "solopoolcom",
    Noderunners => "noderunners",
    Dmnd => "dmnd",
    #[serde(skip)] Dummy172 => "dummy172",
    #[serde(skip)] Dummy173 => "dummy173",
    #[serde(skip)] Dummy174 => "dummy174",
    #[serde(skip)] Dummy175 => "dummy175",
    #[serde(skip)] Dummy176 => "dummy176",
    #[serde(skip)] Dummy177 => "dummy177",
    #[serde(skip)] Dummy178 => "dummy178",
    #[serde(skip)] Dummy179 => "dummy179",
    #[serde(skip)] Dummy180 => "dummy180",
    #[serde(skip)] Dummy181 => "dummy181",
    #[serde(skip)] Dummy182 => "dummy182",
    #[serde(skip)] Dummy183 => "dummy183",
    #[serde(skip)] Dummy184 => "dummy184",
    #[serde(skip)] Dummy185 => "dummy185",
    #[serde(skip)] Dummy186 => "dummy186",
    #[serde(skip)] Dummy187 => "dummy187",
    #[serde(skip)] Dummy188 => "dummy188",
    #[serde(skip)] Dummy189 => "dummy189",
    #[serde(skip)] Dummy190 => "dummy190",
    #[serde(skip)] Dummy191 => "dummy191",
    #[serde(skip)] Dummy192 => "dummy192",
    #[serde(skip)] Dummy193 => "dummy193",
    #[serde(skip)] Dummy194 => "dummy194",
    #[serde(skip)] Dummy195 => "dummy195",
    #[serde(skip)] Dummy196 => "dummy196",
    #[serde(skip)] Dummy197 => "dummy197",
    #[serde(skip)] Dummy198 => "dummy198",
    #[serde(skip)] Dummy199 => "dummy199",
    #[serde(skip)] Dummy200 => "dummy200",
    #[serde(skip)] Dummy201 => "dummy201",
    #[serde(skip)] Dummy202 => "dummy202",
    #[serde(skip)] Dummy203 => "dummy203",
    #[serde(skip)] Dummy204 => "dummy204",
    #[serde(skip)] Dummy205 => "dummy205",
    #[serde(skip)] Dummy206 => "dummy206",
    #[serde(skip)] Dummy207 => "dummy207",
    #[serde(skip)] Dummy208 => "dummy208",
    #[serde(skip)] Dummy209 => "dummy209",
    #[serde(skip)] Dummy210 => "dummy210",
    #[serde(skip)] Dummy211 => "dummy211",
    #[serde(skip)] Dummy212 => "dummy212",
    #[serde(skip)] Dummy213 => "dummy213",
    #[serde(skip)] Dummy214 => "dummy214",
    #[serde(skip)] Dummy215 => "dummy215",
    #[serde(skip)] Dummy216 => "dummy216",
    #[serde(skip)] Dummy217 => "dummy217",
    #[serde(skip)] Dummy218 => "dummy218",
    #[serde(skip)] Dummy219 => "dummy219",
    #[serde(skip)] Dummy220 => "dummy220",
    #[serde(skip)] Dummy221 => "dummy221",
    #[serde(skip)] Dummy222 => "dummy222",
    #[serde(skip)] Dummy223 => "dummy223",
    #[serde(skip)] Dummy224 => "dummy224",
    #[serde(skip)] Dummy225 => "dummy225",
    #[serde(skip)] Dummy226 => "dummy226",
    #[serde(skip)] Dummy227 => "dummy227",
    #[serde(skip)] Dummy228 => "dummy228",
    #[serde(skip)] Dummy229 => "dummy229",
    #[serde(skip)] Dummy230 => "dummy230",
    #[serde(skip)] Dummy231 => "dummy231",
    #[serde(skip)] Dummy232 => "dummy232",
    #[serde(skip)] Dummy233 => "dummy233",
    #[serde(skip)] Dummy234 => "dummy234",
    #[serde(skip)] Dummy235 => "dummy235",
    #[serde(skip)] Dummy236 => "dummy236",
    #[serde(skip)] Dummy237 => "dummy237",
    #[serde(skip)] Dummy238 => "dummy238",
    #[serde(skip)] Dummy239 => "dummy239",
    #[serde(skip)] Dummy240 => "dummy240",
    #[serde(skip)] Dummy241 => "dummy241",
    #[serde(skip)] Dummy242 => "dummy242",
    #[serde(skip)] Dummy243 => "dummy243",
    #[serde(skip)] Dummy244 => "dummy244",
    #[serde(skip)] Dummy245 => "dummy245",
    #[serde(skip)] Dummy246 => "dummy246",
    #[serde(skip)] Dummy247 => "dummy247",
    #[serde(skip)] Dummy248 => "dummy248",
    #[serde(skip)] Dummy249 => "dummy249",
    #[serde(skip)] Dummy250 => "dummy250",
    #[serde(skip)] Dummy251 => "dummy251",
    #[serde(skip)] Dummy252 => "dummy252",
    #[serde(skip)] Dummy253 => "dummy253",
    #[serde(skip)] Dummy254 => "dummy254",
    #[serde(skip)] Dummy255 => "dummy255",
}

impl AsRef<str> for PoolSlug {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for PoolSlug {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(feature = "storage")]
impl Formattable for PoolSlug {
    #[inline]
    fn write_to(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(self.as_str().as_bytes());
    }

    #[inline]
    fn fmt_json(&self, buf: &mut Vec<u8>) {
        buf.push(b'"');
        self.write_to(buf);
        buf.push(b'"');
    }
}

#[cfg(feature = "storage")]
// SAFETY: `Array` is `[u8; size_of::<Self>()]`, so `to_bytes` returns exactly the bytes per
// value the contract requires; the values are encoded, not in native layout.
unsafe impl Bytes for PoolSlug {
    type Array = [u8; size_of::<Self>()];

    #[inline]
    fn to_bytes(&self) -> Self::Array {
        [*self as u8]
    }

    #[inline]
    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != size_of::<Self>() {
            return Err(Error::WrongLength {
                expected: size_of::<Self>(),
                received: bytes.len(),
            });
        };
        Ok(Self::from(bytes[0]))
    }
}

impl From<u8> for PoolSlug {
    #[inline]
    fn from(val: u8) -> Self {
        // SAFETY: PoolSlug is repr(u8) and all 256 values are valid (includes dummy variants)
        unsafe { mem::transmute(val) }
    }
}

impl From<PoolSlug> for u8 {
    #[inline]
    fn from(val: PoolSlug) -> u8 {
        val as u8
    }
}
