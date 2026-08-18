use crate::UserKey;

pub enum Bound {
    Included(UserKey),
    Excluded(UserKey),
}
