/// Series data output format
#[derive(Debug)]
pub enum Output {
    Json(Vec<u8>),
    CSV(Vec<u8>),
}
