use std::mem;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use strum::Display;

#[cfg(feature = "storage")]
use vecdb::Error;
#[cfg(feature = "storage")]
use vecdb::Result;

#[cfg(feature = "storage")]
use vecdb::{Bytes, Formattable};

/// URL-friendly mining pool identifier
#[allow(clippy::upper_case_acronyms)]
#[derive(
    Default,
    Debug,
    Copy,
    Display,
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
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
#[repr(u8)]
pub enum PoolSlug {
    #[default]
    Unknown,
    BlockFills,
    UltimusPool,
    TerraPool,
    Luxor,
    #[serde(rename = "1thash")]
    OneThash,
    BtcCom,
    Bitfarms,
    HuobiPool,
    WayiCn,
    CanoePool,
    BtcTop,
    BitcoinCom,
    #[serde(rename = "175btc")]
    Pool175btc,
    GbMiners,
    AXbt,
    AsicMiner,
    BitMinter,
    BitcoinRussia,
    BtcServ,
    SimplecoinUs,
    BtcGuild,
    Eligius,
    OzCoin,
    EclipseMc,
    MaxBtc,
    TripleMining,
    CoinLab,
    #[serde(rename = "50btc")]
    Pool50btc,
    GhashIo,
    StMiningCorp,
    Bitparking,
    Mmpool,
    Polmine,
    KncMiner,
    Bitalo,
    F2Pool,
    Hhtt,
    MegaBigPower,
    MtRed,
    NmcBit,
    YourbtcNet,
    GiveMeCoins,
    BraiinsPool,
    AntPool,
    MultiCoinCo,
    BcpoolIo,
    Cointerra,
    KanoPool,
    SoloCk,
    CkPool,
    NiceHash,
    BitClub,
    BitcoinAffiliateNetwork,
    Btcc,
    BwPool,
    ExxBw,
    Bitsolo,
    BitFury,
    #[serde(rename = "21inc")]
    TwentyOneInc,
    DigitalBtc,
    #[serde(rename = "8baochi")]
    EightBaochi,
    MyBtcCoinPool,
    TbDice,
    HashPool,
    Nexious,
    BravoMining,
    HotPool,
    OkExPool,
    BcMonster,
    #[serde(rename = "1hash")]
    OneHash,
    Bixin,
    TatmasPool,
    ViaBtc,
    ConnectBtc,
    BatPool,
    Waterhole,
    DcExploration,
    Dcex,
    BtPool,
    #[serde(rename = "58coin")]
    FiftyEightCoin,
    BitcoinIndia,
    ShawnP0wers,
    PHashIo,
    RigPool,
    HaoZhuZhu,
    #[serde(rename = "7pool")]
    SevenPool,
    MiningKings,
    HashBx,
    DPool,
    Rawpool,
    Haominer,
    Helix,
    BitcoinUkraine,
    Poolin,
    SecretSuperstar,
    TigerpoolNet,
    SigmapoolCom,
    OkpoolTop,
    Hummerpool,
    Tangpool,
    BytePool,
    SpiderPool,
    NovaBlock,
    MiningCity,
    BinancePool,
    Minerium,
    LubianCom,
    Okkong,
    AaoPool,
    EmcdPool,
    FoundryUsa,
    SbiCrypto,
    ArkPool,
    PureBtcCom,
    MaraPool,
    KuCoinPool,
    EntrustCharityPool,
    OkMiner,
    Titan,
    PegaPool,
    BtcNuggets,
    CloudHashing,
    DigitalXMintsy,
    Telco214,
    BtcPoolParty,
    Multipool,
    TransactionCoinMining,
    BtcDig,
    TrickysBtcPool,
    BtcMp,
    Eobot,
    Unomp,
    Patels,
    GoGreenLight,
    BitcoinIndiaPool,
    EkanemBtc,
    Canoe,
    Tiger,
    #[serde(rename = "1m1x")]
    OneM1x,
    Zulupool,
    SecPool,
    Ocean,
    WhitePool,
    Wiz,
    #[serde(skip)]
    Dummy145,
    #[serde(skip)]
    Dummy146,
    Wk057,
    FutureBitApolloSolo,
    #[serde(skip)]
    Dummy149,
    #[serde(skip)]
    Dummy150,
    CarbonNegative,
    PortlandHodl,
    Phoenix,
    Neopool,
    MaxiPool,
    #[serde(skip)]
    Dummy156,
    BitFuFuPool,
    GDPool,
    MiningDutch,
    PublicPool,
    MiningSquared,
    InnopolisTech,
    #[serde(skip)]
    Dummy163,
    BtcLab,
    Parasite,
    RedRockPool,
    Est3lar,
    BraiinsSolo,
    #[serde(rename = "solopoolcom")]
    SoloPool,
    Noderunners,
    Dmnd,
    #[serde(skip)]
    Dummy172,
    #[serde(skip)]
    Dummy173,
    #[serde(skip)]
    Dummy174,
    #[serde(skip)]
    Dummy175,
    #[serde(skip)]
    Dummy176,
    #[serde(skip)]
    Dummy177,
    #[serde(skip)]
    Dummy178,
    #[serde(skip)]
    Dummy179,
    #[serde(skip)]
    Dummy180,
    #[serde(skip)]
    Dummy181,
    #[serde(skip)]
    Dummy182,
    #[serde(skip)]
    Dummy183,
    #[serde(skip)]
    Dummy184,
    #[serde(skip)]
    Dummy185,
    #[serde(skip)]
    Dummy186,
    #[serde(skip)]
    Dummy187,
    #[serde(skip)]
    Dummy188,
    #[serde(skip)]
    Dummy189,
    #[serde(skip)]
    Dummy190,
    #[serde(skip)]
    Dummy191,
    #[serde(skip)]
    Dummy192,
    #[serde(skip)]
    Dummy193,
    #[serde(skip)]
    Dummy194,
    #[serde(skip)]
    Dummy195,
    #[serde(skip)]
    Dummy196,
    #[serde(skip)]
    Dummy197,
    #[serde(skip)]
    Dummy198,
    #[serde(skip)]
    Dummy199,
    #[serde(skip)]
    Dummy200,
    #[serde(skip)]
    Dummy201,
    #[serde(skip)]
    Dummy202,
    #[serde(skip)]
    Dummy203,
    #[serde(skip)]
    Dummy204,
    #[serde(skip)]
    Dummy205,
    #[serde(skip)]
    Dummy206,
    #[serde(skip)]
    Dummy207,
    #[serde(skip)]
    Dummy208,
    #[serde(skip)]
    Dummy209,
    #[serde(skip)]
    Dummy210,
    #[serde(skip)]
    Dummy211,
    #[serde(skip)]
    Dummy212,
    #[serde(skip)]
    Dummy213,
    #[serde(skip)]
    Dummy214,
    #[serde(skip)]
    Dummy215,
    #[serde(skip)]
    Dummy216,
    #[serde(skip)]
    Dummy217,
    #[serde(skip)]
    Dummy218,
    #[serde(skip)]
    Dummy219,
    #[serde(skip)]
    Dummy220,
    #[serde(skip)]
    Dummy221,
    #[serde(skip)]
    Dummy222,
    #[serde(skip)]
    Dummy223,
    #[serde(skip)]
    Dummy224,
    #[serde(skip)]
    Dummy225,
    #[serde(skip)]
    Dummy226,
    #[serde(skip)]
    Dummy227,
    #[serde(skip)]
    Dummy228,
    #[serde(skip)]
    Dummy229,
    #[serde(skip)]
    Dummy230,
    #[serde(skip)]
    Dummy231,
    #[serde(skip)]
    Dummy232,
    #[serde(skip)]
    Dummy233,
    #[serde(skip)]
    Dummy234,
    #[serde(skip)]
    Dummy235,
    #[serde(skip)]
    Dummy236,
    #[serde(skip)]
    Dummy237,
    #[serde(skip)]
    Dummy238,
    #[serde(skip)]
    Dummy239,
    #[serde(skip)]
    Dummy240,
    #[serde(skip)]
    Dummy241,
    #[serde(skip)]
    Dummy242,
    #[serde(skip)]
    Dummy243,
    #[serde(skip)]
    Dummy244,
    #[serde(skip)]
    Dummy245,
    #[serde(skip)]
    Dummy246,
    #[serde(skip)]
    Dummy247,
    #[serde(skip)]
    Dummy248,
    #[serde(skip)]
    Dummy249,
    #[serde(skip)]
    Dummy250,
    #[serde(skip)]
    Dummy251,
    #[serde(skip)]
    Dummy252,
    #[serde(skip)]
    Dummy253,
    #[serde(skip)]
    Dummy254,
    #[serde(skip)]
    Dummy255,
}

impl PoolSlug {
    /// Pools with dominance above per-window thresholds get full series.
    /// Thresholds: all-time>=1.0%, 1y>=1.0%, 1m>=0.75%, 1w>=0.5%.
    /// Generated by `scripts/pool_major_threshold.py`.
    pub fn is_major(&self) -> bool {
        matches!(
            self,
            Self::AntPool
                | Self::BinancePool
                | Self::BitFury
                | Self::BraiinsPool
                | Self::BtcCom
                | Self::BtcGuild
                | Self::BtcTop
                | Self::Btcc
                | Self::BwPool
                | Self::Eligius
                | Self::F2Pool
                | Self::FoundryUsa
                | Self::Luxor
                | Self::MaraPool
                | Self::Ocean
                | Self::Poolin
                | Self::SbiCrypto
                | Self::SecPool
                | Self::SpiderPool
                | Self::Unknown
                | Self::ViaBtc
                | Self::WhitePool
        )
    }
}

#[cfg(feature = "storage")]
impl Formattable for PoolSlug {
    fn write_to(&self, buf: &mut Vec<u8>) {
        use std::io::Write;
        write!(buf, "{self}").unwrap();
    }

    fn fmt_json(&self, buf: &mut Vec<u8>) {
        buf.push(b'"');
        self.write_to(buf);
        buf.push(b'"');
    }
}

#[cfg(feature = "storage")]
impl Bytes for PoolSlug {
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
        // SAFETY: PoolId is repr(u8) and we're transmuting from u8
        // All values 0-255 are valid (includes dummy variants)
        let s: Self = unsafe { mem::transmute(bytes[0]) };
        Ok(s)
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
