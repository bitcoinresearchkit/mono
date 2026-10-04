//! Python client generation.
//!
//! This module generates a Python client with type hints for the Bitview API.

mod api;
pub(crate) mod client;
mod tree;
pub(crate) mod types;

use std::{collections::BTreeSet, fmt::Write, io, path::Path};

use super::write_if_changed;
use crate::{Endpoint, IndexSetPattern, TypeSchemas, model::Model};

/// Generate Python client from the series-tree model and OpenAPI endpoints.
///
/// `output_path` is the full path to the output file (e.g., "packages/bitview_client/__init__.py").
pub(crate) fn generate_python_client(
    model: &Model,
    shape_names: &[String],
    accessors: &[IndexSetPattern],
    undefined_types: &BTreeSet<String>,
    endpoints: &[Endpoint],
    schemas: &TypeSchemas,
    output_path: &Path,
) -> io::Result<()> {
    let mut output = String::new();

    writeln!(output, "# Auto-generated Bitview Python client").unwrap();
    writeln!(output, "# Do not edit manually\n").unwrap();
    writeln!(output, "from __future__ import annotations").unwrap();
    writeln!(output, "from dataclasses import dataclass").unwrap();
    writeln!(output, "from functools import cached_property").unwrap();
    writeln!(
        output,
        "from typing import TypeVar, Generic, Any, Callable, Dict, Optional, List, Iterator, Literal, TypedDict, Union, Protocol, overload, Tuple, TYPE_CHECKING"
    )
    .unwrap();
    writeln!(
        output,
        "from http.client import HTTPSConnection, HTTPConnection"
    )
    .unwrap();
    writeln!(output, "from urllib.parse import urlparse").unwrap();
    writeln!(
        output,
        "from datetime import date, datetime, timedelta, timezone"
    )
    .unwrap();
    writeln!(output, "import json").unwrap();
    writeln!(output, "import re\n").unwrap();
    writeln!(output, "if TYPE_CHECKING:").unwrap();
    writeln!(
        output,
        "    import pandas as pd  # type: ignore[import-not-found]"
    )
    .unwrap();
    writeln!(
        output,
        "    import polars as pl  # type: ignore[import-not-found]\n"
    )
    .unwrap();
    writeln!(output, "T = TypeVar('T')\n").unwrap();

    types::generate_type_definitions(&mut output, schemas);
    client::generate_base_client(&mut output);
    client::generate_endpoint_class(&mut output);
    client::generate_index_accessors(&mut output, accessors);
    tree::generate_tree(&mut output, model, shape_names, accessors, undefined_types);
    api::generate_main_client(&mut output, endpoints);

    output.truncate(output.trim_end().len());
    output.push('\n');
    write_if_changed(output_path, &output)
}
