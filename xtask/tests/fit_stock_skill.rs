//! Provenance: repository MCP stock-fit schema and stock-fit refusal rules (#1552).

#[test]
fn fit_stock_skill_documents_the_tool_contract() {
    let guide = include_str!("../../.claude/skills/camcraft-mcp/SKILL.md");
    let row = guide
        .lines()
        .find(|line| line.starts_with("- `fit_stock {"))
        .expect("the MCP skill must list fit_stock in its tool reference");
    for requirement in [
        "\"shape\"?",
        "\"box\"",
        "\"cylinder\"",
        "\"margin\"?",
        "\"sides\"?",
        "default 2",
        "mm",
        "[-X, +X, -Y, +Y, -Z, +Z]",
        "replaces",
        "box only",
        "active setup",
        "parts",
        "never fixtures",
        "rotation",
        "SetStock",
        "One undo",
        "Refused by name",
        "no models",
        "only fixtures",
        "negative",
        "non-finite",
        "six",
        "unknown shape",
        "no usable extent",
    ] {
        assert!(
            row.contains(requirement),
            "fit_stock row must document {requirement}"
        );
    }
}
