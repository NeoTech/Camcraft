//! The MCP post tools (#2074): list the shipped and user posts, select
//! the document's post through the undo history (`SelectPost`), preview
//! a script on a sample or the document stream without writing a file,
//! and save a script of the user's own in the user posts folder.
//!
//! Provenance: our own tool surface over this repo's `SelectPost`
//! command, the post engine (`camcraft_post::post::engine`) and its
//! preview samples (`camcraft_post::post::preview`).

use std::collections::BTreeMap;

use serde_json::{json, Map, Value};

use super::post_files;
use super::setup_tools::{cmd_err, obj};
use super::tools::{opt_str, req_str, Tool};
use crate::commands::SelectPost;
use crate::post::options::OptionValue;
use crate::post::preview::{DEFAULT_VMC_JSON, SAMPLES};
use crate::post::{cnc, engine};
use crate::schema::PostRef;
use crate::script::Env;
use crate::{posting, setup};

pub(super) const TOOLS: &[Tool] = &[
    Tool {
        name: "list_posts",
        description: "List the built-in posts (name, the machine kinds each serves, description, program_units, extension, declared options) and the user's own .cps files in the user posts folder (file, path, the kinds its manifest states or null, the same manifest fields). A file that does not load lists its problem. selected is the document's post.",
        schema: s_empty,
        run: list_posts,
    },
    Tool {
        name: "select_post",
        description: "Select the document's post as one undo step: a built-in post name (see list_posts), a .cps path, or null for the default built-in post. A .cps file must read and run, and a name must be in the registry. A setup's own post is set_setup_post's.",
        schema: s_select_post,
        run: select_post,
    },
    Tool {
        name: "preview_post",
        description: "Run a post script without writing a file, and return the program text. Give the script as source (text) or path (.cps). The stream is a built-in sample (Facing, Contour, Pocket, Drilling; on the default VMC machine) or, without sample, the document's operations on the document's machine. machine names a built-in machine instead. post_options sets declared options of the script. Returns ok, program, lines, warnings and swallowed; a script that fails returns ok: false and errors, each with a message and its 1-based line in the script when the engine states one. The engine's runtime limits stop a script that loops.",
        schema: s_preview_post,
        run: preview_post,
    },
    Tool {
        name: "save_post_script",
        description: "Save a post script as a .cps file in the user posts folder (CAMCRAFT_POSTS, else the per-user data folder camcraft/posts). file is a plain file name, never a path. The script must load and run before it is saved. An existing file is replaced only with overwrite: true. The write is atomic. Returns the path, which select_post and preview_post take.",
        schema: s_save_post_script,
        run: save_post_script,
    },
];

fn s_empty() -> Value {
    obj(json!({}), &[])
}

fn s_select_post() -> Value {
    obj(
        json!({ "post": { "type": ["string", "null"], "description": "A built-in post name, a .cps path, or null for the default built-in post." } }),
        &["post"],
    )
}

fn s_preview_post() -> Value {
    let samples: Vec<&str> = SAMPLES.iter().map(|s| s.name).collect();
    obj(
        json!({
            "source": { "type": "string", "description": "The .cps script text." },
            "path": { "type": "string", "description": "A .cps file to read instead of source." },
            "sample": { "type": "string", "enum": samples, "description": "A built-in sample stream; omitted = the document's operations." },
            "machine": { "type": "string", "description": "A built-in machine name; omitted = the default VMC for a sample, the document's machine for the document." },
            "post_options": { "type": "object", "description": "Values for options that the script declares (declaredOptions)." }
        }),
        &[],
    )
}

fn s_save_post_script() -> Value {
    obj(
        json!({
            "file": { "type": "string", "description": "A plain file name, such as my-post.cps (.cps is added when absent)." },
            "source": { "type": "string", "description": "The .cps script text." },
            "overwrite": { "type": "boolean", "description": "Replace an existing file of that name (default false)." }
        }),
        &["file", "source"],
    )
}

fn user_folder(env: &Env) -> Option<std::path::PathBuf> {
    env.user_posts.clone().or_else(post_files::user_dir)
}

fn list_posts(env: &mut Env, _a: &Map<String, Value>) -> Result<Value, String> {
    let builtin: Vec<Value> = posting::BUILTINS.iter().map(|(n, _)| post_files::builtin_row(n)).collect();
    let folder = user_folder(env);
    let (user, problem) = match folder.as_deref().map(post_files::user_files) {
        None => (Vec::new(), Some(format!("no user posts folder (set {} or a home folder)", post_files::ENV_VAR))),
        Some(Ok(files)) => (files.iter().map(|p| post_files::user_row(p)).collect(), None),
        Some(Err(why)) => (Vec::new(), Some(why)),
    };
    Ok(json!({
        "selected": posting::post_ref_label(&posting::document_post_of(&env.world)),
        "builtin": builtin,
        "user_folder": folder.map(|p| p.display().to_string()),
        "user": user,
        "user_problem": problem,
    }))
}

fn select_post(env: &mut Env, a: &Map<String, Value>) -> Result<Value, String> {
    let next = match a.get("post") {
        None => return Err("\"post\" is required (a built-in post name, a .cps path, or null for the default)".into()),
        Some(Value::Null) => PostRef::Builtin,
        Some(Value::String(s)) if s.trim().is_empty() => return Err("\"post\" must not be empty (null selects the default)".into()),
        Some(Value::String(s)) if s.trim().to_ascii_lowercase().ends_with(".cps") => PostRef::File(s.trim().to_owned()),
        Some(Value::String(s)) => PostRef::Named(s.trim().to_owned()),
        Some(other) => return Err(format!("\"post\" must be a string or null, got {other}")),
    };
    let previous = posting::post_ref_label(&posting::document_post_of(&env.world));
    let cmd = SelectPost::new(&env.world, next).map_err(cmd_err)?;
    env.history.execute(&env.world, cmd);
    Ok(json!({ "post": posting::post_ref_label(&posting::document_post_of(&env.world)), "previous": previous }))
}

fn script_text(a: &Map<String, Value>) -> Result<String, String> {
    match (opt_str(a, "source")?, opt_str(a, "path")?) {
        (Some(s), None) => Ok(s.to_owned()),
        (None, Some(p)) => crate::session_files::read_post(p).map_err(|e| format!("post script {p}: {e}")),
        _ => Err("give exactly one of \"source\" or \"path\"".into()),
    }
}

fn option_overrides(a: &Map<String, Value>) -> Result<Option<BTreeMap<String, OptionValue>>, String> {
    let Some(raw) = a.get("post_options") else {
        return Ok(None);
    };
    let map = raw.as_object().ok_or("\"post_options\" must be an object")?;
    let mut out = BTreeMap::new();
    for (k, v) in map {
        let parsed: OptionValue = serde_json::from_value(v.clone()).map_err(|_| format!("invalid value for post option '{k}'"))?;
        out.insert(k.clone(), parsed);
    }
    Ok(Some(out))
}

fn machine_of(env: &Env, a: &Map<String, Value>, sample: bool) -> Result<Value, String> {
    let text = match opt_str(a, "machine")? {
        Some(name) => posting::builtin_machine_text(name)?,
        None if sample => DEFAULT_VMC_JSON,
        None => return Ok(posting::machine_value(&env.world)),
    };
    serde_json::from_str(text).map_err(|e| format!("machine file: {e}"))
}

/// The 1-based line of the script that an engine message names, when
/// it names one (`user.cps:3:5`, or `line 3`).
fn error_line(msg: &str) -> Option<u64> {
    let digits = |s: &str| s.chars().take_while(char::is_ascii_digit).collect::<String>().parse::<u64>().ok();
    if let Some(i) = msg.find("user.cps:") {
        return digits(&msg[i + "user.cps:".len()..]);
    }
    msg.find("line ").and_then(|i| digits(&msg[i + "line ".len()..]))
}

fn preview_post(env: &mut Env, a: &Map<String, Value>) -> Result<Value, String> {
    let cps = script_text(a)?;
    let sample = match opt_str(a, "sample")? {
        None => None,
        Some(n) => Some(SAMPLES.iter().find(|s| s.name.eq_ignore_ascii_case(n)).ok_or_else(|| {
            let names: Vec<&str> = SAMPLES.iter().map(|s| s.name).collect();
            format!("no sample named '{n}' - samples: {}", names.join(", "))
        })?),
    };
    let stream = match sample {
        Some(s) => s.stream().map_err(|e| format!("sample {}: {e}", s.name))?,
        None => cnc::serialize(&posting::stream_records(&env.world)?),
    };
    let mch = machine_of(env, a, sample.is_some())?;
    let extra = option_overrides(a)?;
    let active = setup::active(&env.world);
    let fail = |msg: String| json!({ "ok": false, "errors": [{ "message": msg, "line": error_line(&msg) }] });
    let opts = match posting::program_options_for_script(&env.world, &cps, None, active.as_deref(), extra.as_ref()) {
        Ok(o) => o,
        Err(why) => return Ok(fail(why)),
    };
    Ok(match engine::run_with(&cps, &mch, &stream, &opts) {
        Ok(out) => json!({
            "ok": true,
            "program": out.program,
            "lines": out.line_map.len(),
            "warnings": out.warnings,
            "swallowed": out.swallowed,
        }),
        Err(e) => fail(e.to_string()),
    })
}

fn save_post_script(env: &mut Env, a: &Map<String, Value>) -> Result<Value, String> {
    let file = req_str(a, "file")?;
    let source = req_str(a, "source")?;
    let overwrite = match a.get("overwrite") {
        None | Some(Value::Null) => false,
        Some(Value::Bool(b)) => *b,
        Some(other) => return Err(format!("\"overwrite\" must be true or false, got {other}")),
    };
    let dir = user_folder(env).ok_or_else(|| format!("no user posts folder (set {} or a home folder)", post_files::ENV_VAR))?;
    let (path, replaced) = post_files::save(&dir, file, source, overwrite)?;
    Ok(json!({ "path": path.display().to_string(), "replaced": replaced, "bytes": source.len() }))
}

#[cfg(test)]
mod tests {
    use super::error_line;

    #[test]
    fn the_error_line_reads_the_script_position() {
        assert_eq!(error_line("SyntaxError: bad token at user.cps:3:5"), Some(3));
        assert_eq!(error_line("post script: unexpected token at line 12, col 1"), Some(12));
        assert_eq!(error_line("post script: onOpen threw"), None);
    }
}
