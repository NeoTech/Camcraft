//! The MCP holder file tools (#4389): write the document's user holders
//! to a holder file, and import a holder file as user holders in one
//! undo step - the same `ImportHolders` command as the Holders page's
//! Import… button.
//! Provenance: our own tool surface over our own component library
//! format (#3645).

use serde_json::{Map, Value, json};

use super::component_library_tools::{full_library, library_prop};
use super::setup_tools::{cmd_err, obj};
use super::tools::{Tool, req_str};
use crate::commands::{ImportHolders, export_holders as holder_text};
use crate::script::Env;

pub(super) const TOOLS: &[Tool] = &[
    Tool {
        name: "export_holders",
        description: "Write the document's user holders (each holder in the component table that is not built in) to a holder file, in the component library format (camcraft-components/1). A file write, not a document change. Refused by name: a document with no user holder, and an unwriteable path.",
        schema: s_export,
        run: export_holders,
    },
    Tool {
        name: "import_holders",
        description: "Import a holder file (camcraft-components/1) as user holders of the document, in one undo step. An id that the document or the component library holds already takes a new user:holder:<name> id. Refused by name, with no change: an unreadable file, an unreadable component library (its ids cannot be kept clear), every component library file refusal, a file with no component, and a component that is not a holder.",
        schema: s_import,
        run: import_holders,
    },
];

fn s_export() -> Value {
    obj(json!({ "path": { "type": "string", "description": "The holder file to write (.camcomponents)." } }), &["path"])
}

fn s_import() -> Value {
    obj(
        json!({
            "path": { "type": "string", "description": "The holder file to read (.camcomponents)." },
            "library": library_prop(),
        }),
        &["path"],
    )
}

fn export_holders(env: &mut Env, a: &Map<String, Value>) -> Result<Value, String> {
    let path = req_str(a, "path")?.trim();
    let text = holder_text(&env.world)?;
    std::fs::write(path, text).map_err(|e| format!("holders not exported to {path}: {e}"))?;
    let n = crate::commands::holder_file::user_holders(&env.world).len();
    Ok(json!({ "path": path, "holders": n }))
}

fn import_holders(env: &mut Env, a: &Map<String, Value>) -> Result<Value, String> {
    let path = req_str(a, "path")?.trim();
    let text = std::fs::read_to_string(path).map_err(|e| format!("holder file {path} is not read: {e}"))?;
    // A new id keeps clear of the library ids. An unreadable library is
    // refused by name: without its ids, a new id could equal a library id.
    let library = full_library(a)
        .map_err(|e| format!("holders not imported: the component library is not read, so its ids are not kept clear: {e}"))?;
    let reserved: Vec<String> = library.iter().map(|c| c.component.id().to_owned()).collect();
    let cmd = ImportHolders::from_text(&env.world, &text, &reserved).map_err(cmd_err)?;
    let imported: Vec<Value> = cmd.imported().iter().map(|i| json!({ "file_id": i.file_id, "id": i.id })).collect();
    let summary = cmd.summary();
    env.history.execute(&env.world, cmd);
    Ok(json!({ "imported": imported, "summary": summary }))
}
