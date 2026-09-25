//! Schema-driven parameter validation and cache-key helpers.

use std::collections::HashMap;

use regex::Regex;

use crate::{
    errors::{AppError, BadRequest, NotFound},
    models::{Field, FieldKind, UniversityInfo},
    universities::Params,
};

const MAX_VALUE_LEN: usize = 100;

pub fn all_fields(info: &UniversityInfo) -> impl Iterator<Item = &Field> {
    info.steps.iter().flat_map(|s| s.fields.iter())
}

pub fn find_field<'a>(info: &'a UniversityInfo, key: &str) -> Result<&'a Field, AppError> {
    all_fields(info)
        .find(|f| f.key == key)
        .ok_or_else(|| NotFound::new(format!("Unknown field '{key}' for '{}'", info.id)).into())
}

/// A non-empty query parameter.
pub fn required<'a>(query: &'a HashMap<String, String>, key: &str) -> Result<&'a String, AppError> {
    query
        .get(key)
        .filter(|v| !v.is_empty())
        .ok_or_else(|| BadRequest::new(format!("Missing required parameter '{key}'")).into())
}

/// Checks that don't need network access: length, static options, remote `pattern`.
pub fn validate_shallow(field: &Field, value: &str) -> Result<(), AppError> {
    let invalid = || BadRequest::new(format!("Invalid value for '{}'", field.key)).into();
    if value.len() > MAX_VALUE_LEN {
        return Err(invalid());
    }
    match &field.kind {
        FieldKind::Select { options, .. } if !options.iter().any(|o| o.value == value) => {
            Err(invalid())
        }
        FieldKind::RemoteSelect { .. } => match field.pattern {
            Some(pattern) if !Regex::new(pattern).is_ok_and(|re| re.is_match(value)) => {
                Err(invalid())
            }
            _ => Ok(()),
        },
        _ => Ok(()),
    }
}

/// `a=1&b=2` with keys sorted (Params is a BTreeMap).
pub fn canonical(params: &Params) -> String {
    params
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join("&")
}
