//! The face names of a Solidcraft design's final body (#3915): SolidCraft's
//! own name for each face of the body record, read from the model cache
//! beside the body, one per record face and in record order. Only model
//! cache format 10 is read. Names never refuse an import: names that do not
//! fit the body are dropped with one note, and the import goes on as before.
//! Provenance: SolidCraft's published source (tangency-labs/solidcraft,
//! Apache-2.0) at v0.50.0. crates/doc/src/model_cache.rs writes and reads
//! the list `model.names`: one record `{faces, info}` for each distinct body,
//! a model body's `names` field its index, one `info` item for each face,
//! `[aliases, bounds, verified, synonyms]`, and a fifth item, the former
//! name of a Split Face piece. crates/kernel/src/body.rs (`Body::record`,
//! "in the body's own order"; `face_count`), crates/kernel/src/native.rs
//! (`encode_native_body` writes that record) and crates/doc/src/
//! history_names.rs (one name for each face) give the face order. The
//! container is docs/design-container.md, "Model cache". No layout was
//! worked out from sample bytes; no HSMWorks material consulted.

use serde_json::Value;

/// The model-cache format whose face names are read
/// (docs/decisions/4212-sc-face-names.md, D1).
pub const NAMES_FORMAT: &str = "solidcraft-model-cache/10";

/// The longest face name kept, in bytes: our own bound. SolidCraft's names
/// are short ("F2:side:l4"); a longer one is damage, and the list is dropped.
pub const MAX_NAME_BYTES: usize = 1 << 14;

/// One record face's SolidCraft name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceName {
    /// The face's name (`model.names[n].faces[i]`).
    pub name: String,
    /// The name follows exact history (`info[i][2]`); `false`: SolidCraft
    /// worked it out from geometry.
    pub verified: bool,
    /// The name a Split Face piece had before (`info[i][4]`, SolidCraft
    /// 0.50.0 and later); `None` when the item is not there.
    pub former: Option<String>,
}

/// The names of the final body's `faces` record faces, for the model body
/// whose `names` field is `entry`. `Ok(None)`: the cache is not format 10,
/// so its names are not read (no note). `Ok(Some(names))`: one name for
/// each record face, in record order. `Err(note)`: a format-10 cache whose
/// names do not fit; the caller keeps the note and imports without names.
pub(crate) fn read(
    manifest: &Value,
    entry: Option<&Value>,
    faces: usize,
) -> Result<Option<Vec<SourceName>>, String> {
    if manifest.get("format").and_then(Value::as_str) != Some(NAMES_FORMAT) {
        return Ok(None);
    }
    let not = |why: String| format!("the face names were not read: {why}");
    let list = manifest
        .get("model")
        .and_then(|m| m.get("names"))
        .and_then(Value::as_array)
        .ok_or_else(|| not("the model cache has no names list".into()))?;
    let at = entry
        .and_then(Value::as_u64)
        .and_then(|i| usize::try_from(i).ok())
        .ok_or_else(|| not("the final body names no names record".into()))?;
    let record = list.get(at).ok_or_else(|| {
        not(format!(
            "names record {at} is past the {} in the list",
            list.len()
        ))
    })?;
    let texts = record
        .get("faces")
        .and_then(Value::as_array)
        .ok_or_else(|| not(format!("names record {at} has no face list")))?;
    let info = record
        .get("info")
        .and_then(Value::as_array)
        .ok_or_else(|| not(format!("names record {at} has no info list")))?;
    if texts.len() != faces || info.len() != faces {
        return Err(not(format!(
            "{} names and {} info items for the body's {faces} faces",
            texts.len(),
            info.len()
        )));
    }
    let mut out = Vec::new();
    out.try_reserve_exact(faces)
        .map_err(|e| not(format!("no memory for {faces} names ({e})")))?;
    for (i, (text, item)) in texts.iter().zip(info).enumerate() {
        let name =
            name_text(Some(text)).ok_or_else(|| not(format!("name {i} is not a short text")))?;
        let (verified, former) = info_item(item).ok_or_else(|| {
            not(format!(
                "info {i} is not [aliases, bounds, verified, synonyms] with an optional former name"
            ))
        })?;
        out.push(SourceName {
            name,
            verified,
            former,
        });
    }
    Ok(Some(out))
}

/// A face name: a text of 1 to [`MAX_NAME_BYTES`] bytes.
fn name_text(v: Option<&Value>) -> Option<String> {
    v.and_then(Value::as_str)
        .filter(|s| !s.is_empty() && s.len() <= MAX_NAME_BYTES)
        .map(str::to_owned)
}

/// `(verified, former)` of one info item: four items, or five where the
/// fifth is a name (or null, which SolidCraft's own reader takes as none).
/// Items 0, 1 and 3 must be lists; their contents are not read.
fn info_item(item: &Value) -> Option<(bool, Option<String>)> {
    let items = item.as_array()?;
    if !matches!(items.len(), 4 | 5) {
        return None;
    }
    let lists = [0, 1, 3]
        .iter()
        .all(|i| items.get(*i).is_some_and(Value::is_array));
    let verified = items.get(2).and_then(Value::as_bool)?;
    let former = match items.get(4) {
        None | Some(Value::Null) => None,
        Some(v) => Some(name_text(Some(v))?),
    };
    lists.then_some((verified, former))
}
