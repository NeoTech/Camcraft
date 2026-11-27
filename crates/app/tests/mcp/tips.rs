//! Tips (#3030): the MCP door reads the same repository file as Help.
//! Gate map: covers app::mcp::tips, app::mcp::tools, app::mcp::fallback.
//! Provenance: our own tests over the public MCP JSON-RPC surface.

use super::*;

#[test]
fn list_tips_is_listed_and_read_only() {
    let mut c = Client::new();
    let listed = c.request("tools/list", json!({}));
    assert!(
        listed["result"]["tools"]
            .as_array()
            .unwrap()
            .iter()
            .any(|t| t["name"] == "list_tips")
    );
    let before = c.ok("get_job", json!({}));
    let tips = c.ok("list_tips", json!({}));
    assert_eq!(tips["tips"].as_array().unwrap().len(), 4);
    assert_eq!(c.ok("get_job", json!({})), before);
    assert!(camcraft_app::mcp::provider::is_read_only_tool("list_tips"));
    let (error, message) = c.call("list_tips", json!({"typo":true}));
    assert!(error && message.as_str().unwrap().contains("unknown argument"));
}

#[test]
fn tips_match_the_file_have_stable_ids_and_existing_guide_links() {
    use camcraft_app::mcp::tips::{self, Tip};
    let source: Vec<Tip> =
        serde_json::from_str(include_str!("../../../../assets/tips/tips.json")).unwrap();
    assert_eq!(tips::all().unwrap(), source);
    let ids: Vec<_> = source.iter().map(|t| t.id.as_str()).collect();
    assert_eq!(
        ids,
        [
            "mcp-setup",
            "undo-ai-changes",
            "verify-before-post",
            "example-catalog"
        ]
    );
    let mut c = Client::new();
    let response = c.ok("list_tips", json!({}));
    let rendered: Vec<_> = source
        .iter()
        .map(|t| t.for_os(std::env::consts::OS))
        .collect();
    assert_eq!(response["tips"], serde_json::to_value(rendered).unwrap());
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for tip in &source {
        assert!(!tip.title.is_empty() && !tip.text.is_empty());
        let url = tip
            .guide_link
            .as_ref()
            .expect("first four tips have working links");
        let target = if let Some(page) = url.strip_prefix("https://neotech.github.io/Camcraft/") {
            root.join("site/content")
                .join(page.trim_end_matches('/'))
                .with_extension("md")
        } else {
            let path = url
                .strip_prefix("https://github.com/NeoTech/Camcraft/tree/main/")
                .expect("approved repository link");
            root.join(path).join("README.md")
        };
        assert!(target.is_file(), "{}: {}", tip.id, target.display());
    }
    assert!(source[0].text.contains("MCP") && source[0].text.contains("headless"));
    assert!(source[1].text.contains("AI agent") && source[1].text.contains("undo history"));
    assert!(source[2].text.contains("refusal") && source[2].text.contains("dry run"));
    assert!(source[3].text.contains("example catalog"));
    for os in ["windows", "linux", "macos"] {
        let rendered = source[1].for_os(os);
        assert!(
            rendered
                .text
                .contains(if os == "macos" { "Cmd+Z" } else { "Ctrl+Z" })
        );
        assert!(
            !rendered
                .text
                .contains(if os == "macos" { "Ctrl+Z" } else { "Cmd+Z" })
        );
        assert!(!rendered.text.contains("{undo}"));
    }
}
