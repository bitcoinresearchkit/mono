use std::path::PathBuf;

#[derive(Clone, Debug)]
pub enum Auth {
    None,
    UserPass(String, String),
    CookieFile(PathBuf),
}
