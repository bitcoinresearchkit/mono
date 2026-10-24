//! Where a response's object keys are data, not field names: the schema's maps, whose keys the
//! JavaScript runtime leaves as sent (`_camelKeys`'s `keep`) while it camelCases field names.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

use crate::{Ty, TypeSchemas};

/// How the runtime treats a value's keys.
#[derive(PartialEq)]
enum Keep {
    /// Every key is a field name: camelCased.
    Convert,
    /// A map: its keys and everything under it as sent.
    All,
    /// A struct whose listed fields (wire names) hold maps.
    Fields(BTreeMap<String, Keep>),
}

/// The runtime's `keep` for a response of type `ty`, as a JavaScript expression; `None` when every
/// key is a field name. Panics when a struct with snake_case fields sits under a map: its keys
/// would stay as sent while its JavaScript type promises camelCase.
pub(crate) fn keep_expr(ty: &Ty, schemas: &TypeSchemas) -> Option<String> {
    match keep(ty, schemas, &mut BTreeSet::new()) {
        Keep::Convert => None,
        keep => Some(render(&keep)),
    }
}

fn keep(ty: &Ty, schemas: &TypeSchemas, visiting: &mut BTreeSet<String>) -> Keep {
    match ty {
        Ty::Map(_) => under_map(ty, schemas),
        Ty::Union(variants) if variants.iter().any(|variant| matches!(variant, Ty::Map(_))) => {
            under_map(ty, schemas)
        }
        Ty::Union(variants) => variants
            .iter()
            .map(|variant| keep(variant, schemas, visiting))
            .fold(Keep::Convert, merge),
        Ty::List(item) | Ty::Nullable(item) => keep(item, schemas, visiting),
        Ty::Ref(name) => {
            // A struct reached again through itself holds no map on this path.
            let Some(schema) = schemas.get(name).filter(|_| visiting.insert(name.clone())) else {
                return Keep::Convert;
            };
            let keep = match properties(schema) {
                Some(properties) => {
                    let fields: BTreeMap<String, Keep> = properties
                        .iter()
                        .map(|(key, field)| {
                            (key.clone(), keep(&Ty::parse(field), schemas, visiting))
                        })
                        .filter(|(_, keep)| *keep != Keep::Convert)
                        .collect();
                    if fields.is_empty() {
                        Keep::Convert
                    } else {
                        Keep::Fields(fields)
                    }
                }
                None => keep(&Ty::parse(schema), schemas, visiting),
            };
            visiting.remove(name);
            keep
        }
        _ => Keep::Convert,
    }
}

/// A map: its keys stay, and so does everything under it, so no struct there may have
/// snake_case fields.
fn under_map(ty: &Ty, schemas: &TypeSchemas) -> Keep {
    assert!(
        !has_snake_fields(ty, schemas, &mut BTreeSet::new()),
        "a struct with snake_case fields under a map ({ty}): its keys would stay as sent"
    );
    Keep::All
}

fn has_snake_fields(ty: &Ty, schemas: &TypeSchemas, visiting: &mut BTreeSet<String>) -> bool {
    match ty {
        Ty::Map(value) | Ty::List(value) | Ty::Nullable(value) => {
            has_snake_fields(value, schemas, visiting)
        }
        Ty::Union(variants) | Ty::Tuple(variants) => variants
            .iter()
            .any(|variant| has_snake_fields(variant, schemas, visiting)),
        Ty::Ref(name) => {
            let Some(schema) = schemas.get(name).filter(|_| visiting.insert(name.clone())) else {
                return false;
            };
            match properties(schema) {
                Some(properties) => properties.iter().any(|(key, field)| {
                    renamed(key) || has_snake_fields(&Ty::parse(field), schemas, visiting)
                }),
                None => has_snake_fields(&Ty::parse(schema), schemas, visiting),
            }
        }
        _ => false,
    }
}

/// Whether the runtime renames `key` (`tx_count`, not `avgFee_0`).
fn renamed(key: &str) -> bool {
    key.as_bytes()
        .windows(2)
        .any(|pair| pair[0] == b'_' && pair[1].is_ascii_lowercase())
}

fn properties(schema: &Value) -> Option<&serde_json::Map<String, Value>> {
    schema.get("properties").and_then(Value::as_object)
}

fn merge(a: Keep, b: Keep) -> Keep {
    match (a, b) {
        (Keep::Convert, keep) | (keep, Keep::Convert) => keep,
        (Keep::All, _) | (_, Keep::All) => Keep::All,
        (Keep::Fields(mut a), Keep::Fields(b)) => {
            for (key, keep) in b {
                let merged = match a.remove(&key) {
                    Some(existing) => merge(existing, keep),
                    None => keep,
                };
                a.insert(key, merged);
            }
            Keep::Fields(a)
        }
    }
}

fn render(keep: &Keep) -> String {
    match keep {
        Keep::Convert => "undefined".to_owned(),
        Keep::All => "true".to_owned(),
        Keep::Fields(fields) => {
            let fields: Vec<String> = fields
                .iter()
                .map(|(key, keep)| {
                    format!("{}: {}", serde_json::to_string(key).unwrap(), render(keep))
                })
                .collect();
            format!("{{ {} }}", fields.join(", "))
        }
    }
}
