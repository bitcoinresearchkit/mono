use bitview_plugin::{Plugin, PluginId, PluginStorage};
use bitview_transforms::{ReturnF32Tenths, ReturnI8, ReturnU16};
use bitview_traversable::Traversable;
use bitview_vecs::{ConstantVecs, IndexSources};
use brk_types::{StoredF32, StoredI8, StoredU16, Version};

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("constants"), Version::new(9));
pub const ID: PluginId = STORAGE.id();

#[derive(Clone, Traversable)]
pub struct Vecs {
    /// Constant numeric value 0 at every supported index.
    _0: ConstantVecs<StoredU16>,
    /// Constant numeric value 1 at every supported index.
    _1: ConstantVecs<StoredU16>,
    /// Constant numeric value 2 at every supported index.
    _2: ConstantVecs<StoredU16>,
    /// Constant numeric value 3 at every supported index.
    _3: ConstantVecs<StoredU16>,
    /// Constant numeric value 4 at every supported index.
    _4: ConstantVecs<StoredU16>,
    /// Constant numeric value 20 at every supported index.
    _20: ConstantVecs<StoredU16>,
    /// Constant numeric value 30 at every supported index.
    _30: ConstantVecs<StoredU16>,
    /// Constant numeric value 38.2 at every supported index.
    _38_2: ConstantVecs<StoredF32>,
    /// Constant numeric value 50 at every supported index.
    _50: ConstantVecs<StoredU16>,
    /// Constant numeric value 61.8 at every supported index.
    _61_8: ConstantVecs<StoredF32>,
    /// Constant numeric value 70 at every supported index.
    _70: ConstantVecs<StoredU16>,
    /// Constant numeric value 80 at every supported index.
    _80: ConstantVecs<StoredU16>,
    /// Constant numeric value 100 at every supported index.
    _100: ConstantVecs<StoredU16>,
    /// Constant numeric value 600 at every supported index.
    _600: ConstantVecs<StoredU16>,
    /// Constant numeric value -1 at every supported index.
    _minus_1: ConstantVecs<StoredI8>,
    /// Constant numeric value -2 at every supported index.
    _minus_2: ConstantVecs<StoredI8>,
    /// Constant numeric value -3 at every supported index.
    _minus_3: ConstantVecs<StoredI8>,
    /// Constant numeric value -4 at every supported index.
    _minus_4: ConstantVecs<StoredI8>,
}

impl Vecs {
    pub fn new(indexes: &IndexSources) -> Self {
        let version = STORAGE.schema_version();
        Self {
            _0: ConstantVecs::new::<ReturnU16<0>>("constant_0", version, indexes),
            _1: ConstantVecs::new::<ReturnU16<1>>("constant_1", version, indexes),
            _2: ConstantVecs::new::<ReturnU16<2>>("constant_2", version, indexes),
            _3: ConstantVecs::new::<ReturnU16<3>>("constant_3", version, indexes),
            _4: ConstantVecs::new::<ReturnU16<4>>("constant_4", version, indexes),
            _20: ConstantVecs::new::<ReturnU16<20>>("constant_20", version, indexes),
            _30: ConstantVecs::new::<ReturnU16<30>>("constant_30", version, indexes),
            _38_2: ConstantVecs::new::<ReturnF32Tenths<382>>("constant_38_2", version, indexes),
            _50: ConstantVecs::new::<ReturnU16<50>>("constant_50", version, indexes),
            _61_8: ConstantVecs::new::<ReturnF32Tenths<618>>("constant_61_8", version, indexes),
            _70: ConstantVecs::new::<ReturnU16<70>>("constant_70", version, indexes),
            _80: ConstantVecs::new::<ReturnU16<80>>("constant_80", version, indexes),
            _100: ConstantVecs::new::<ReturnU16<100>>("constant_100", version, indexes),
            _600: ConstantVecs::new::<ReturnU16<600>>("constant_600", version, indexes),
            _minus_1: ConstantVecs::new::<ReturnI8<-1>>("constant_minus_1", version, indexes),
            _minus_2: ConstantVecs::new::<ReturnI8<-2>>("constant_minus_2", version, indexes),
            _minus_3: ConstantVecs::new::<ReturnI8<-3>>("constant_minus_3", version, indexes),
            _minus_4: ConstantVecs::new::<ReturnI8<-4>>("constant_minus_4", version, indexes),
        }
    }
}

impl Plugin for Vecs {
    fn storage(&self) -> PluginStorage {
        STORAGE
    }
}
