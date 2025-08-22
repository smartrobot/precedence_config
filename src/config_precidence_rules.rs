use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

/// Incoming/outgoing Matrix row (wide) with dynamic attribute keys.
/// Expecting JSON like:
///  ```JSON
/// [
/// { "rank": 1, "col_1": 1, "col_2": 1, "col_3": 1 },
/// { "rank": 2, "col_1": 0, "col_2": 1, "col_3": 1 },
/// { "rank": 3, "col_1": 0, "col_2": 0, "col_3": 1 }
/// ]
/// ```

#[derive(Debug, Deserialize, Serialize)]
pub struct MatrixRow {
    pub rank: i32,
    #[serde(flatten)]
    pub attrs: HashMap<String, u8>,
}


/// Canonical Tall row (normalized)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ConfigPrecedenceRule {
    pub config_version_id: i32,
    pub rank: i32,
    pub attr_id: i32,
    pub match_type: u8, // 0, 1
}

use anyhow::{anyhow, bail, Context, Result};

/// Converts matrix-style JSON into tall rows with resolved attr_ids.
pub fn matrix_json_to_tall(
    json: &str,
    config_version_id: i32,
    attr_name_to_id: &HashMap<String, i32>,
) -> Result<Vec<ConfigPrecedenceRule>> {
    let matrix_rows: Vec<MatrixRow> = serde_json::from_str(json)
        .with_context(|| "Invalid JSON: expected an array of objects with `rank` and attributes")?;

    let mut tall = Vec::new();
    let mut seen = HashSet::new();

    for row in matrix_rows {
        if row.rank <= 0 {
            bail!("Rank must be >= 1 (found {})", row.rank);
        }

        for (attr_name, match_type) in row.attrs.iter() {
            let Some(&attr_id) = attr_name_to_id.get(attr_name) else {
                continue; // or bail! if strict
            };

            if *match_type > 1 {
                bail!("MATCH_TYPE must be 0 or 1 (found {} for attr {})", match_type, attr_name);
            }

            let key = (row.rank, attr_id);
            if !seen.insert(key) {
                bail!("Duplicate (rank, attr_id): ({}, {})", row.rank, attr_id);
            }

            tall.push(ConfigPrecedenceRule {
                config_version_id,
                rank: row.rank,
                attr_id,
                match_type: *match_type,
            });
        }
    }

    if tall.is_empty() {
        bail!("No valid precedence rules parsed from JSON");
    }

    Ok(tall)
}


/// Converts tall precedence rules into matrix-style rows using attr_id → attr_name mapping.
pub fn tall_to_matrix_rows(
    tall: &[ConfigPrecedenceRule],
    attr_id_to_name: &HashMap<i32, String>,
) -> Result<Vec<MatrixRow>> {
    let mut by_rank: BTreeMap<i32, BTreeMap<String, u8>> = BTreeMap::new();
    let mut seen = HashSet::new();

    for r in tall {
        if r.rank <= 0 {
            bail!("Rank must be positive starting at 1 (found {})", r.rank);
        }
        if r.match_type > 1 {
            bail!("MATCH_TYPE must be 0 or 1 (found {} for attr_id {})", r.match_type, r.attr_id);
        }

        let Some(attr_name) = attr_id_to_name.get(&r.attr_id) else {
            continue; // or bail! if strict
        };

        let key = (r.rank, attr_name.clone());
        if !seen.insert(key.clone()) {
            bail!("Duplicate (rank, attr_name): ({}, {})", key.0, key.1);
        }

        by_rank.entry(r.rank)
            .or_default()
            .insert(attr_name.clone(), r.match_type);
    }

    if by_rank.is_empty() {
        bail!("No rows to emit");
    }

    let mut out = Vec::with_capacity(by_rank.len());
    for (rank, attrs) in by_rank {
        out.push(MatrixRow {
            rank,
            attrs: attrs.into_iter().collect(),
        });
    }

    Ok(out)
}



/// Validate ranks are exactly 1 SUM_ATTR with no gaps using a triangular sum check.

/// Validate that ranks are exactly 1..=T(A) with no gaps,
/// where A = number of attributes (sum_attr).
pub fn validate_ranks_contiguous_and_triangular(tall: &[ConfigPrecedenceRule], attr_count: usize) -> Result<()> {
    if tall.is_empty() {
        return Err(anyhow!("No rows provided"));
    }

    // T(A) ranks expected for A attributes
    let t_a: i32 = (attr_count as i32) * ((attr_count as i32) + 1) / 2;

    // Collect distinct ranks
    let ranks: BTreeSet<i32> = tall.iter().map(|r| r.rank).collect();

    // Check count matches T(A)
    if ranks.len() as i32 != t_a {
        return Err(anyhow!(
            "Triangular count check failed: found {} distinct ranks, expected {} for A={}",
            ranks.len(),
            t_a,
            attr_count
        ));
    }

    // Check contiguity: ranks must be exactly 1..=T(A)
    let mut expected = 1;
    for r in &ranks {
        if *r != expected {
            return Err(anyhow!(
                "Contiguity check failed at rank {}: expected {}",
                r,
                expected
            ));
        }
        expected += 1;
    }

    Ok(())
}

