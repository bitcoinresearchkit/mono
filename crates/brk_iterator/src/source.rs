use brk_reader::Reader;
use brk_rpc::Client;

pub(crate) struct Source {
    pub(crate) client: Client,
    pub(crate) reader: Reader,
}
