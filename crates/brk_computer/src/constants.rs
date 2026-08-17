use brk_plugin::{Plugin, PluginGate};
use brk_traversable::Traversable;
use brk_types::{StoredF32, StoredI8, StoredU16, Version};

use super::{
    indexes,
    internal::{ConstantVecs, ReturnF32Tenths, ReturnI8, ReturnU16},
};

pub const DB_NAME: &str = "constants";

#[derive(Clone, Traversable)]
pub struct Vecs {
    #[traversable(skip)]
    pub(crate) plugin_gate: PluginGate,
    /// Constant numeric value 0 at every supported index.
    pub _0: ConstantVecs<StoredU16>,
    /// Constant numeric value 1 at every supported index.
    pub _1: ConstantVecs<StoredU16>,
    /// Constant numeric value 2 at every supported index.
    pub _2: ConstantVecs<StoredU16>,
    /// Constant numeric value 3 at every supported index.
    pub _3: ConstantVecs<StoredU16>,
    /// Constant numeric value 4 at every supported index.
    pub _4: ConstantVecs<StoredU16>,
    /// Constant numeric value 20 at every supported index.
    pub _20: ConstantVecs<StoredU16>,
    /// Constant numeric value 30 at every supported index.
    pub _30: ConstantVecs<StoredU16>,
    /// Constant numeric value 38.2 at every supported index.
    pub _38_2: ConstantVecs<StoredF32>,
    /// Constant numeric value 50 at every supported index.
    pub _50: ConstantVecs<StoredU16>,
    /// Constant numeric value 61.8 at every supported index.
    pub _61_8: ConstantVecs<StoredF32>,
    /// Constant numeric value 70 at every supported index.
    pub _70: ConstantVecs<StoredU16>,
    /// Constant numeric value 80 at every supported index.
    pub _80: ConstantVecs<StoredU16>,
    /// Constant numeric value 100 at every supported index.
    pub _100: ConstantVecs<StoredU16>,
    /// Constant numeric value 600 at every supported index.
    pub _600: ConstantVecs<StoredU16>,
    /// Constant numeric value -1 at every supported index.
    pub _minus_1: ConstantVecs<StoredI8>,
    /// Constant numeric value -2 at every supported index.
    pub _minus_2: ConstantVecs<StoredI8>,
    /// Constant numeric value -3 at every supported index.
    pub _minus_3: ConstantVecs<StoredI8>,
    /// Constant numeric value -4 at every supported index.
    pub _minus_4: ConstantVecs<StoredI8>,
}

impl Plugin for Vecs {
    fn id(&self) -> &'static str {
        DB_NAME
    }

    fn gate(&self) -> &PluginGate {
        &self.plugin_gate
    }
}

impl Vecs {
    pub(crate) fn new(version: Version, indexes: &indexes::Vecs) -> Self {
        let v = version;

        Self {
            plugin_gate: Default::default(),
            _0: ConstantVecs::new::<ReturnU16<0>>("constant_0", v, indexes),
            _1: ConstantVecs::new::<ReturnU16<1>>("constant_1", v, indexes),
            _2: ConstantVecs::new::<ReturnU16<2>>("constant_2", v, indexes),
            _3: ConstantVecs::new::<ReturnU16<3>>("constant_3", v, indexes),
            _4: ConstantVecs::new::<ReturnU16<4>>("constant_4", v, indexes),
            _20: ConstantVecs::new::<ReturnU16<20>>("constant_20", v, indexes),
            _30: ConstantVecs::new::<ReturnU16<30>>("constant_30", v, indexes),
            _38_2: ConstantVecs::new::<ReturnF32Tenths<382>>("constant_38_2", v, indexes),
            _50: ConstantVecs::new::<ReturnU16<50>>("constant_50", v, indexes),
            _61_8: ConstantVecs::new::<ReturnF32Tenths<618>>("constant_61_8", v, indexes),
            _70: ConstantVecs::new::<ReturnU16<70>>("constant_70", v, indexes),
            _80: ConstantVecs::new::<ReturnU16<80>>("constant_80", v, indexes),
            _100: ConstantVecs::new::<ReturnU16<100>>("constant_100", v, indexes),
            _600: ConstantVecs::new::<ReturnU16<600>>("constant_600", v, indexes),
            _minus_1: ConstantVecs::new::<ReturnI8<-1>>("constant_minus_1", v, indexes),
            _minus_2: ConstantVecs::new::<ReturnI8<-2>>("constant_minus_2", v, indexes),
            _minus_3: ConstantVecs::new::<ReturnI8<-3>>("constant_minus_3", v, indexes),
            _minus_4: ConstantVecs::new::<ReturnI8<-4>>("constant_minus_4", v, indexes),
        }
    }
}
