use vecdb::StorageMode;

use crate::Vecs;

pub trait HasSilentPayments<M: StorageMode> {
    fn silent_payments(&self) -> &Vecs<M>;
}
