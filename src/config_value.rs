use chrono::NaiveDateTime;
use anyhow::{bail, Result};
use std::collections::HashMap;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct ConfigValue {
    pub match_id: i32,
    pub attr_id: i32,
    pub role: String, // "match" or "param"
    pub value: TypedValue,
}

#[derive(Debug, Clone)]
pub enum TypedValue {
    Int(i64),
    Dec(f64),
    Str(String),
    Bool(bool),
    Dt(NaiveDateTime),
}

#[derive(Debug, Clone)]
pub struct AttrMeta {
    pub attr_id: i32,
    pub attr_name: String,
    pub data_type: String, // "int", "dec", "str", "bool", "dt"
    pub role: String,      // "match" or "param"
}

#[derive(Debug, Deserialize)]
pub struct RawParam {
    pub key: String,
    pub type_: String, // "int", "dec", etc.
    pub value: String,
}

/// ```JSON
/// {
///     "match_id": 123,
///     "params": [
///         { "key": "discount_pct", "type": "dec", "value": "0.125" },
///         { "key": "max_orders_day", "type": "int", "value": 250 }
///     ]
/// }
/// ```
pub fn parse_config_values(
    match_id: i32,
    raw_params: &[RawParam],
    attr_lookup: &HashMap<String, AttrMeta>,
) -> Result<Vec<ConfigValue>> {
    let mut out = Vec::new();

    for param in raw_params {
        let Some(meta) = attr_lookup.get(&param.key) else {
            bail!("Unknown attribute key: {}", param.key);
        };

        if meta.role != "param" {
            bail!("Attribute '{}' is not a param (role = {})", param.key, meta.role);
        }

        let value = match meta.data_type.as_str() {
            "int" => {
                let v = param.value.parse::<i64>()?;
                TypedValue::Int(v)
            }
            "dec" => {
                let v = param.value.parse::<f64>()?;
                TypedValue::Dec(v)
            }
            "str" => TypedValue::Str(param.value.clone()),
            "bool" => {
                let v = param.value.parse::<bool>()?;
                TypedValue::Bool(v)
            }
            "dt" => {
                let v = NaiveDateTime::parse_from_str(&param.value, "%Y-%m-%dT%H:%M:%SZ")?;
                TypedValue::Dt(v)
            }
            _ => bail!("Unsupported data type: {}", meta.data_type),
        };

        out.push(ConfigValue {
            match_id,
            attr_id: meta.attr_id,
            role: meta.role.clone(),
            value,
        });
    }

    Ok(out)
}
