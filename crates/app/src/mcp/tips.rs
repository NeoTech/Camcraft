//! Shared Help/MCP tips (#3030), from our repository's tips file.
//! Provenance: our own onboarding text, checked against the shipped guide.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use std::sync::OnceLock;

use crate::script::Env;

/// One repository tip. `{undo}` in the text is resolved for the user's OS.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Tip {
    pub id: String,
    pub title: String,
    pub text: String,
    pub guide_link: Option<String>,
}

impl Tip {
    /// The same text for UI and MCP, with the platform's undo shortcut.
    #[must_use]
    pub fn for_os(&self, os: &str) -> Self {
        let undo = if os == "macos" { "Cmd+Z" } else { "Ctrl+Z" };
        Self {
            text: self.text.replace("{undo}", undo),
            ..self.clone()
        }
    }
}

/// Read once, without runtime asset paths. A broken bundled file is a named
/// refusal, not a panic or an empty successful response.
pub fn all() -> Result<&'static [Tip], &'static str> {
    static TIPS: OnceLock<Result<Vec<Tip>, String>> = OnceLock::new();
    TIPS.get_or_init(|| {
        serde_json::from_str(include_str!("../../../../assets/tips/tips.json"))
            .map_err(|e| format!("tips file not read: {e}"))
    })
    .as_ref()
    .map(Vec::as_slice)
    .map_err(String::as_str)
}

pub(super) const TOOLS: &[super::tools::Tool] = &[super::tools::Tool {
    name: "list_tips",
    description: "Read the Help tips: stable ids, titles, text and optional guide links. Uses this server's OS for keyboard shortcuts. Does not change the document or undo history.",
    schema: || json!({"type":"object", "properties":{}, "additionalProperties":false}),
    run: list_tips,
}];

fn list_tips(_env: &mut Env, _args: &Map<String, Value>) -> Result<Value, String> {
    let tips: Vec<_> = all()
        .map_err(str::to_owned)?
        .iter()
        .map(|t| t.for_os(std::env::consts::OS))
        .collect();
    Ok(json!({"tips":tips}))
}
