// Copyright (c) 2024-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

use crate::FORMAT_VERSION;

pub const DATABASE_FORMAT: &[u8] = &[b'F', b'J', b'L', FORMAT_VERSION];
pub const KEYSPACES_FOLDER: &str = "keyspaces";

pub const LOCK_FILE: &str = "lock";
pub const VERSION_MARKER: &str = "version";
