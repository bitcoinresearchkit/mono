use std::{
    borrow::Cow,
    fs::File,
    io::{self, BufRead, BufReader},
    sync::OnceLock,
};

use axum::http::{HeaderMap, HeaderValue, header::ETAG};
use bitcoin::hex::DisplayHex;
#[cfg(feature = "full-api")]
use bitview_query::RepresentationId;
use sha2::{Digest, Sha256};

static BUILD_TAG: OnceLock<io::Result<Cow<'static, str>>> = OnceLock::new();

/// Use an application build's source digest for ETags. Call before starting the server.
pub fn set_build_tag(tag: &'static str) {
    BUILD_TAG.set(Ok(Cow::Borrowed(tag))).expect("ETag build tag is already set");
}

/// For applications without a source digest, hash the executable before startup work.
pub fn initialize() -> io::Result<()> {
    build_tag().as_ref().map(|_| ()).map_err(|error| {
        io::Error::new(error.kind(), format!("Hashing the running executable for ETags: {error}"))
    })
}

fn build_tag() -> &'static io::Result<Cow<'static, str>> {
    BUILD_TAG.get_or_init(|| hash_executable().map(Cow::Owned))
}

fn hash_executable() -> io::Result<String> {
    // On Linux, open the running image even if an install replaced its path.
    #[cfg(target_os = "linux")]
    let file = File::open("/proc/self/exe")?;
    #[cfg(not(target_os = "linux"))]
    let file = File::open(std::env::current_exe()?)?;
    let mut reader = BufReader::with_capacity(128 * 1024, file);
    let mut hash = Sha256::new();
    loop {
        let bytes = reader.fill_buf()?;
        if bytes.is_empty() {
            break;
        }
        hash.update(bytes);
        let len = bytes.len();
        reader.consume(len);
    }
    Ok(hash.finalize()[..8].to_lower_hex_string())
}

/// A weak ETag prefixed with the build's tag so code can change response formats
/// without a manual format version on each tag.
#[derive(Clone, Debug)]
pub struct Etag(HeaderValue);

impl Etag {
    /// Names bytes by their hash: `l{hash:x}`.
    pub fn content(hash: u64) -> Self {
        format!("l{hash:x}").into()
    }

    /// Names a representation the way it identifies itself: by its content, or by the block that
    /// fixes it.
    #[cfg(feature = "full-api")]
    pub fn representation(identity: RepresentationId) -> Self {
        match identity {
            RepresentationId::Content(hash) => Self::content(hash),
            RepresentationId::Block(hash) => hash.to_string().into(),
        }
    }

    pub fn matches(&self, headers: &HeaderMap) -> bool {
        bitview_http::if_none_match(headers, Some(self.token()))
    }

    pub fn insert(&self, headers: &mut HeaderMap) {
        headers.insert(ETAG, self.0.clone());
    }

    fn token(&self) -> &[u8] {
        let value = self.0.as_bytes();
        &value[3..value.len() - 1]
    }
}

impl From<String> for Etag {
    fn from(value: String) -> Self {
        debug_assert!(
            value
                .bytes()
                .all(|byte| byte == 0x21 || (0x23..=0x7e).contains(&byte) || byte >= 0x80),
            "invalid ETag token"
        );
        let prefix = build_tag().as_ref().expect("ETag prefix initialized at startup");
        let mut header = String::with_capacity(prefix.len() + value.len() + 5);
        header.push_str("W/\"");
        header.push_str(prefix);
        header.push('-');
        header.push_str(&value);
        header.push('"');
        Self(HeaderValue::try_from(header).unwrap())
    }
}
