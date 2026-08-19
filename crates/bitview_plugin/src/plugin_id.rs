use std::fmt;

/// Stable identity of a Bitview plugin.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PluginId(&'static str);

impl PluginId {
    pub const fn new(id: &'static str) -> Self {
        let bytes = id.as_bytes();
        assert!(!bytes.is_empty(), "plugin ID cannot be empty");
        let mut index = 0;
        while index < bytes.len() {
            let byte = bytes[index];
            assert!(
                byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_' || byte == b'-',
                "plugin ID must be a safe directory name"
            );
            index += 1;
        }
        Self(id)
    }

    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

impl fmt::Display for PluginId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}
