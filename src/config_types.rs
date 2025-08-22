use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct ConfigEnvelope {
    pub config: ConfigMeta,
    pub rows: Vec<ConfigRow>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ConfigMeta {
    pub name: String,
    pub version: i32,
    pub version_name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ConfigRow {
    #[serde(flatten)]
    pub match_part: MatchPart,
    pub params: Vec<Param>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MatchPart {
    // All dynamic match attributes live here
    #[serde(rename = "match")]
    pub attrs: HashMap<String, serde_json::Value>, // allow null/number/string
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Param {
    pub key: String,
    #[serde(rename = "type")]
    pub ty: ParamType,
    pub value: serde_json::Value, // validated downstream based on ty
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ParamType {
    Int,
    Dec,
    Str,
    Bool,
    Dt,
}

