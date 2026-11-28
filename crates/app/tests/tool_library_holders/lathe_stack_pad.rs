//! #3643 review (PR #4323): an assembled turning tool whose holder body
//! has a pad (a meshed body's chord deviation) keeps the grown clearance
//! bands of its effective body through a save and an open, and a second
//! save is byte-stable. A wrong pad on the body refuses the file by name.
//! Provenance: our own tests over our own commands and format.

use camcraft_app::commands::{AddTool, History, SetToolAssembly};
use camcraft_app::model;
use camcraft_app::tool_body::{BodyFrame, body_of};
use flecs_ecs::prelude::World;

use super::lathe_stack::{A, holder_m_padded, insert, reopen, stack, turning, upsert, HOLDER_M};

/// A PCLNR holder (its body grown by `pad`) with a CNMG insert, as tool `A`.
fn assembled(pad: f64) -> World {
    let w = camcraft_app::world::new_world();
    let mut h = History::default();
    for row in [holder_m_padded(pad), insert("CNMG120408")] {
        upsert(&w, &mut h, row);
    }
    h.execute(&w, AddTool::new(&w, A, turning()).expect("tool"));
    h.execute(&w, SetToolAssembly::new(&w, A, stack(HOLDER_M)).expect("assembly"));
    w
}

/// The effective body's clearance words and pad.
fn judged(w: &World) -> (Vec<f64>, f64) {
    let e = model::find_by_name(w, A).expect("tool");
    let body = body_of(w, &e).expect("a lathe body");
    let BodyFrame::Lathe { clearance, pad, .. } = &body.frame else { panic!("a lathe-frame body") };
    (clearance.words().collect(), *pad)
}

/// The one lathe-placed tool body object in a saved file.
fn tool_body(v: &mut serde_json::Value) -> Option<&mut serde_json::Value> {
    match v {
        serde_json::Value::Object(m) if m.contains_key("lathe") && m.contains_key("facets") => Some(v),
        serde_json::Value::Object(m) => m.values_mut().find_map(tool_body),
        serde_json::Value::Array(a) => a.iter_mut().find_map(tool_body),
        _ => None,
    }
}

#[test]
fn an_assembled_padded_lathe_tool_keeps_its_bands_through_save_and_open() {
    let pad = 0.05;
    let w = assembled(pad);
    let before = judged(&w);
    assert_eq!(before.1, pad, "the effective body keeps the stack's pad");
    assert_ne!(before.0, judged(&assembled(0.0)).0, "the pad grows the bands");

    let text = camcraft_app::project::save_text(&w).expect("save");
    let mut file: serde_json::Value = serde_json::from_str(&text).expect("json");
    let body = tool_body(&mut file).expect("the tool's lathe body on file");
    assert_eq!(body["pad"].as_f64(), Some(pad), "the effective body writes its pad");

    let w2 = reopen(&text);
    assert_eq!(judged(&w2), before, "the same bands and pad after open");
    assert_eq!(camcraft_app::project::save_text(&w2).expect("save"), text, "save -> open -> save is byte-stable");

    // A wrong pad on the body refuses the file by name; nothing smaller opens.
    // A null pad is not read as no pad.
    for bad in [serde_json::json!(-0.05), serde_json::Value::Null, serde_json::json!("0.05")] {
        let mut v = file.clone();
        if let Some(b) = tool_body(&mut v) {
            b["pad"] = bad.clone();
        }
        let w3 = camcraft_app::world::new_world();
        let Err(e) = camcraft_app::project::open(&w3, &v.to_string()) else { panic!("a pad of {bad} opened") };
        let e = e.to_string();
        // A negative pad is refused by its name and the tool's; a pad
        // that is no number is refused by the JSON reader.
        assert!(!bad.is_number() || (e.contains(A) && e.contains("pad")), "a pad of {bad}: {e}");
    }
    // A pad on a spindle body is refused by name too.
    let mut v = file.clone();
    if let Some(b) = tool_body(&mut v).and_then(serde_json::Value::as_object_mut) {
        b.remove("lathe");
        b.insert("axis".into(), serde_json::json!([0.0, 0.0, 1.0]));
        b.insert("offset".into(), serde_json::json!(0.0));
    }
    let w4 = camcraft_app::world::new_world();
    let Err(e) = camcraft_app::project::open(&w4, &v.to_string()) else { panic!("a spindle body with a pad opened") };
    assert!(e.to_string().contains("pad"), "{e}");
}
