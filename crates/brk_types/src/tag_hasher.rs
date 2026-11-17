use std::hash::Hasher;

use rapidhash::v3::{DEFAULT_RAPID_SECRETS, RapidStreamHasherV3, rapidhash_v3};

/// The hash behind every validator and content token the server gives: rapidhash v3 under its
/// published fixed secrets, so equal content gets an equal hash on every server and across
/// restarts (a client may revalidate against any of them). Its output is specified and portable,
/// 13x faster than SipHash on a 1 MB body, and incremental writes hash as their concatenation. Not
/// keyed: content chosen to collide could make two representations share a tag, which nothing
/// gains from; maps over chosen keys keep their own random keys.
pub struct TagHasher(RapidStreamHasherV3<'static>);

/// [`TagHasher`] over one byte string.
pub fn tag_hash(bytes: &[u8]) -> u64 {
    rapidhash_v3(bytes)
}

impl Default for TagHasher {
    fn default() -> Self {
        Self(RapidStreamHasherV3::new(&DEFAULT_RAPID_SECRETS))
    }
}

impl Hasher for TagHasher {
    fn write(&mut self, bytes: &[u8]) {
        self.0.write(bytes);
    }

    fn finish(&self) -> u64 {
        self.0.finish()
    }
}
