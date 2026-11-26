//! Drift pins for the agent docs (#1947): every tool name that a skill,
//! a wrapper, the guide or the local prompt names is a live MCP tool,
//! and every live tool is documented in the generated reference and the
//! guide.
//!
//! Gate map: covers app::mcp::tools, app::mcp::catalog,
//! app::mcp::provider, app::mcp::fallback.

use camcraft_app::mcp::catalog;
use std::path::PathBuf;

/// The agent-facing docs that name MCP tools, from the repo root. The
/// generated reference is pinned whole by its own test.
const DOCS: [&str; 4] = [
    ".claude/skills/camcraft-mcp/SKILL.md",
    ".agents/skills/camcraft-mcp/SKILL.md",
    "GEMINI.md",
    "site/content/guide/automate/mcp.md",
];
const GUIDE: &str = "site/content/guide/automate/mcp.md";

#[test]
fn the_drift_suite_maps_the_tool_table_catalog_and_prompt_context() {
    let source = read("crates/app/tests/mcp_doc_drift.rs");
    let header = source.lines().take_while(|line| line.starts_with("//!")).collect::<Vec<_>>();
    let map = header
        .iter()
        .skip_while(|line| !line.contains("Gate map:"))
        .take_while(|line| line.trim() != "//!")
        .copied()
        .collect::<Vec<_>>()
        .join(" ");
    for module in ["tools", "catalog", "provider", "fallback"] {
        let module = format!("app::mcp::{module}");
        assert!(
            map.split(|c: char| !c.is_ascii_alphanumeric() && c != '_' && c != ':')
                .any(|name| name == module),
            "the drift suite Gate map must cover {module}"
        );
    }
}

fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").join(rel);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
        .replace("\r\n", "\n")
}

fn is_tool_word(w: &str) -> bool {
    w.contains('_')
        && w.starts_with(|c: char| c.is_ascii_lowercase())
        && w.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

/// Tool calls written as `name {` - after a backtick or at a line start.
fn called_tools(text: &str) -> Vec<String> {
    let mut found = Vec::new();
    for line in text.lines() {
        for (i, chunk) in line.split('`').enumerate() {
            // A code span (odd chunk) or a whole line in a fenced block.
            let candidate = if i % 2 == 1 || i == 0 { chunk.trim_start() } else { continue };
            if let Some((word, _)) = candidate.split_once(" {")
                && is_tool_word(word)
            {
                found.push(word.to_owned());
            }
        }
    }
    found
}

/// Backticked tool-shaped words in Markdown table rows.
fn table_tools(text: &str) -> Vec<String> {
    text.lines()
        .filter(|l| l.starts_with('|'))
        .flat_map(|l| l.split('`').skip(1).step_by(2).map(str::to_owned).collect::<Vec<_>>())
        .filter(|w| is_tool_word(w))
        .collect()
}

#[test]
fn the_scanner_finds_a_tool_call_and_a_table_name() {
    assert_eq!(called_tools("- `no_such_tool { \"a\": 1 }`: x"), ["no_such_tool"]);
    assert_eq!(called_tools("no_such_tool {}\n"), ["no_such_tool"]);
    assert_eq!(table_tools("| Area | `no_such_tool`, `get_job` |"), ["no_such_tool", "get_job"]);
}

#[test]
fn the_committed_reference_matches_the_live_tool_table() {
    let committed = read(catalog::REFERENCE_PATH);
    assert!(
        committed == catalog::reference_markdown(),
        "{} is out of date with the MCP tool table. Write it again with:\n{}",
        catalog::REFERENCE_PATH,
        catalog::REFERENCE_COMMAND
    );
}

#[test]
fn every_tool_named_in_the_agent_docs_is_live() {
    let live = catalog::tool_names();
    let mut dead = Vec::new();
    for doc in DOCS {
        let text = read(doc);
        for name in called_tools(&text).into_iter().chain(table_tools(&text)) {
            if !live.contains(&name.as_str()) {
                dead.push(format!("{doc}: {name}"));
            }
        }
    }
    assert!(dead.is_empty(), "docs name tools that the MCP server does not list:\n{}", dead.join("\n"));
}

#[test]
fn every_live_tool_is_in_the_guide_and_the_local_prompt() {
    let guide = read(GUIDE);
    let prompt = catalog::local_prompt();
    let missing: Vec<_> = catalog::tool_names()
        .into_iter()
        .filter(|n| !guide.contains(&format!("`{n}`")) && !guide.contains(&format!("`{n} {{")))
        .collect();
    assert!(missing.is_empty(), "{GUIDE} does not name these live tools: {}", missing.join(", "));
    for name in catalog::tool_names() {
        assert!(prompt.contains(&format!("\n- {name}(")), "local prompt lacks {name}");
    }
}

/// Stop at the next section or separator, not at an unrelated tool row.
fn known_gaps(skill: &str) -> String {
    let (_, rest) = skill
        .split_once("## Known Remaining Gaps\n")
        .expect("the skill keeps its Known Remaining Gaps section");
    rest.lines()
        .take_while(|line| {
            let line = line.trim();
            line != "---" && !line.starts_with("## ") && !line.starts_with("# ")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Recognize snake_case and CamelCase identifiers, plus single-word tools
/// in code spans. Ordinary prose such as "post a follow-up" is not a tool.
fn gap_tool_mentions(text: &str) -> Vec<String> {
    let mut names = Vec::new();
    for (i, span) in text.split('`').enumerate() {
        for word in span.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_')) {
            let mut snake = String::new();
            for (i, c) in word.chars().enumerate() {
                if c.is_ascii_uppercase() && i > 0 {
                    snake.push('_');
                }
                snake.push(c.to_ascii_lowercase());
            }
            if is_tool_word(word) || is_tool_word(&snake) || i % 2 == 1 {
                names.push(snake);
            }
        }
    }
    names
}

#[test]
fn the_gap_scanner_distinguishes_identifiers_from_prose_and_other_sections() {
    let skill = "## Known Remaining Gaps\nset_units, SetProgram and `post` are identifiers.\n\
                 A post or undo in prose is not an identifier. `NoSuchCommand` is not live.\n\
                 ## Shipped\n`get_job`\n";
    let names = gap_tool_mentions(&known_gaps(skill));
    let live = catalog::tool_names();
    let found: Vec<_> = names
        .iter()
        .map(String::as_str)
        .filter(|name| live.contains(name))
        .collect();
    assert_eq!(found, ["set_units", "set_program", "post"]);
    assert_eq!(
        known_gaps("## Known Remaining Gaps\nmissing\n---\n`get_job`"),
        "missing"
    );
}

#[test]
fn the_known_gaps_name_no_shipped_tool() {
    let names = gap_tool_mentions(&known_gaps(&read(DOCS[0])));
    let shipped: Vec<_> = catalog::tool_names()
        .into_iter()
        .filter(|name| names.iter().any(|word| word == name))
        .collect();
    assert!(
        shipped.is_empty(),
        "the Known Remaining Gaps list names shipped tools: {}",
        shipped.join(", ")
    );
}

#[test]
fn the_skill_links_the_generated_reference_and_local_prompt() {
    let skill = read(DOCS[0]);
    let link = format!("](../../../{})", catalog::REFERENCE_PATH);
    assert!(
        skill.contains(&link),
        "the skill must link to {}",
        catalog::REFERENCE_PATH
    );
    for command in ["camcraft-cli mcp reference", "camcraft-cli mcp prompt"] {
        assert!(
            skill.contains(&format!("`{command}`")),
            "the skill must document {command}"
        );
    }
}

#[test]
fn the_local_prompt_json_context_carries_the_shared_rules() {
    let context = camcraft_app::mcp::provider::prompt_context(&serde_json::json!({}));
    let instructions = context["instructions"].as_str().unwrap_or_default();
    assert!(instructions.starts_with(catalog::RULES), "{instructions}");
    assert!(catalog::local_prompt().starts_with(catalog::RULES));
}
