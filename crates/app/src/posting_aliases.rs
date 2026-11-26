//! Hidden alias names of renamed built-in posts (#3537, #4000). Provenance:
//! CamCraft's own post registry history; no list shows an alias name.
//!
//! A saved project (document post, setup post, option key `builtin:<name>`),
//! a script `cps:`, an MCP or CLI post name and a machine `controller.post`
//! that still say an old name select the registry name. A save writes the
//! registry name.

use crate::schema::PostRef;

/// `(old name, registry name)`, one row for each old name of a built-in post.
pub const POST_ALIASES: &[(&str, &str)] = &[
    // #3537 renamed it; #4000 renamed it again.
    ("Agie native wire EDM", super::agie::POST),
    // #4000: the name between #3537 and #4000.
    ("Native wire EDM job files", super::agie::POST),
];

/// The registry name that `name` selects: the registry name for an alias, else `name`.
#[must_use]
pub fn current_name(name: &str) -> &str {
    POST_ALIASES
        .iter()
        .find(|(old, _)| *old == name)
        .map_or(name, |(_, new)| *new)
}

/// `post` with an alias name replaced by the registry name it selects.
#[must_use]
pub fn current_ref(post: PostRef) -> PostRef {
    match post {
        PostRef::Named(name) => PostRef::Named(current_name(&name).to_owned()),
        other => other,
    }
}

/// An option key `builtin:<alias>` rewritten to the key of the registry name.
#[must_use]
pub fn current_key(key: &str) -> String {
    match key.strip_prefix("builtin:") {
        Some(name) => format!("builtin:{}", current_name(name)),
        None => key.to_owned(),
    }
}
