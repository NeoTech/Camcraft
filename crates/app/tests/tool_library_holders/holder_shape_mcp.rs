//! #4391: `edit_user_holder`, the MCP door to the holder form (#4338),
//! driven only through MCP over a shrink-fit holder that carries one
//! indexable cutter. Pinned: a shape edit lands as one undo step that
//! derives the tool again, and undo and redo restore it; the outline is
//! `HolderShape::geometry` of the stated shape; the refusals (a value that
//! cannot be steel, a broken fit, a holder that is not a shape outline,
//! an unknown clamp, an edit that changes nothing) name their reason and
//! change nothing.
//! Provenance: our own tests over our own tools.

use camcraft_app::component::{ADAPTIVE_PORT, ToolComponent};
use camcraft_app::holder_library::{BT40, HolderShape, ShapeClamp};
use camcraft_app::mcp::Server;
use camcraft_app::project::{component_to_dto, tool_to_json};
use camcraft_app::schema::DocumentComponent;
use camcraft_app::tools::{Tool, ToolKind};
use serde_json::{Value, json};

use super::assembly_cmds::{A, INSERT, fly, holder, insert};
use super::{call, cat};

const ID: &str = "user:holder:Shrink 20";

struct Mcp {
    s: Server,
    id: i64,
}

impl Mcp {
    fn ok(&mut self, name: &str, args: Value) -> Value {
        self.id += 1;
        let (err, body) = call(&mut self.s, self.id, name, args);
        assert!(!err, "{name}: {body}");
        body
    }

    fn refused(&mut self, name: &str, args: Value) -> String {
        self.id += 1;
        let (err, body) = call(&mut self.s, self.id, name, args);
        assert!(err, "{name} must be refused: {body}");
        body.as_str().unwrap_or_default().to_owned()
    }
}

fn shrink20() -> HolderShape {
    HolderShape { taper: BT40, clamp: ShapeClamp::Shrink, bore: 20.0, gauge_len: 80.0, nose_d: 32.0, largest_d: 44.0 }
}

/// The document row of `shape`, as `upsert_tool_component` takes it.
fn shaped(shape: &HolderShape) -> Value {
    let kind = shape.kind().expect("kind");
    let c = ToolComponent::new(ID, "Shrink 20", kind).expect("holder");
    let g = shape.geometry(ADAPTIVE_PORT).expect("geometry");
    serde_json::to_value(component_to_dto(&DocumentComponent::new(c, g))).expect("dto")
}

/// The shrink holder, a fly cutter on it and the plain holder of the
/// #3644 shop (its outline is not a shape outline).
fn shop() -> Mcp {
    let mut m = Mcp { s: Server::new(), id: 0 };
    m.s.handle(&json!({ "jsonrpc": "2.0", "id": 0, "method": "initialize", "params": { "protocolVersion": "2025-06-18" } }));
    m.ok("upsert_tool_component", json!({ "component": shaped(&shrink20()) }));
    for c in [holder(), fly("fly-25", 12.5), insert("CCMT09T304")] {
        let dto = serde_json::to_value(component_to_dto(&DocumentComponent::new(c.0, c.1))).expect("dto");
        m.ok("upsert_tool_component", json!({ "component": dto }));
    }
    let cutter = Tool { kind: ToolKind::BullNose, diameter: 20.0, corner_r: 0.2, flute_len: 5.0, shank_d: 20.0, ..cat("EM-1200-2FL") };
    m.ok("add_tool", json!({ "name": A, "definition": tool_to_json(&cutter), "holder": "none" }));
    let links = json!([
        { "component": ID },
        { "component": "fly-25", "mount": { "parent": 0 } },
        { "component": INSERT, "mount": { "parent": 1, "port": "CSW1" } },
    ]);
    m.ok("set_tool_assembly", json!({ "tool": A, "links": links }));
    m
}

/// The document row `ID` as `list_tool_components` lists it.
fn row(m: &mut Mcp) -> Value {
    let rows = m.ok("list_tool_components", json!({ "scope": "document", "kind": "adaptive" }));
    rows["components"].as_array().expect("rows").iter().find(|r| r["component"]["id"] == json!(ID)).expect("the holder row")["component"].clone()
}

fn gauge(m: &mut Mcp) -> f64 {
    m.ok("get_tool_assembly", json!({ "tool": A }))["gauge_len"].as_f64().expect("gauge_len")
}

#[test]
fn mcp_edits_a_user_holder_by_its_shape_in_one_undo_step() {
    let mut m = shop();
    let before_row = row(&mut m);
    let before_gauge = gauge(&mut m);
    let r = m.ok("edit_user_holder", json!({ "id": ID, "gauge_len": 90.0, "largest_d": 48.0 }));
    assert_eq!(r["updated_tools"], json!([A]), "the tool on the holder is derived again: {r}");
    assert_eq!(r["shape"], json!({ "taper": "BT40", "clamp": "shrink", "bore": 20.0, "gauge_len": 90.0, "nose_d": 32.0, "largest_d": 48.0 }));
    let want = HolderShape { gauge_len: 90.0, largest_d: 48.0, ..shrink20() };
    assert_eq!(row(&mut m), shaped(&want), "the outline is HolderShape::geometry of the stated shape");
    assert!((gauge(&mut m) - before_gauge - 10.0).abs() < 1e-9, "the tool follows the holder's gauge length");

    m.ok("undo", json!({}));
    assert_eq!(row(&mut m), before_row, "one undo step restores the holder");
    assert!((gauge(&mut m) - before_gauge).abs() < 1e-9, "and the tool");
    m.ok("redo", json!({}));
    assert_eq!(row(&mut m), shaped(&want), "redo lands the edit again");

    let r = m.ok("edit_user_holder", json!({ "id": ID, "name": "Shrink 20 long" }));
    assert_eq!(r["name"], json!("Shrink 20 long"));
    assert_eq!(r["shape"]["gauge_len"], json!(90.0), "an unstated field keeps the holder's value");
}

#[test]
fn mcp_edit_user_holder_refuses_by_name_and_changes_nothing() {
    let mut m = shop();
    let before_row = row(&mut m);
    let before = m.ok("get_tool_assembly", json!({ "tool": A }));

    let e = m.refused("edit_user_holder", json!({ "id": ID, "nose_d": 10.0 }));
    assert!(e.contains("holder not changed") && e.contains("must be larger than the Ø20 mm it clamps"), "{e}");
    let e = m.refused("edit_user_holder", json!({ "id": ID, "largest_d": 30.0 }));
    assert!(e.contains("widens from its nose"), "{e}");
    let e = m.refused("edit_user_holder", json!({ "id": ID, "gauge_len": -1.0 }));
    assert!(e.contains("the gauge length must be a positive number"), "{e}");
    let e = m.refused("edit_user_holder", json!({ "id": ID, "bore": 25.0, "nose_d": 40.0, "largest_d": 50.0 }));
    assert!(e.contains(ID) && e.contains(A) && e.contains("does not fit"), "a broken fit names the tool: {e}");
    let e = m.refused("edit_user_holder", json!({ "id": "bt40-cyl20", "gauge_len": 60.0 }));
    assert!(e.contains("'bt40-cyl20'") && e.contains("upsert_tool_component"), "{e}");
    assert!(m.refused("edit_user_holder", json!({ "id": "nope" })).contains("no tool component 'nope'"));
    assert!(m.refused("edit_user_holder", json!({ "id": ID, "clamp": "ER99" })).contains("unknown clamp 'ER99'"));
    assert!(m.refused("edit_user_holder", json!({ "id": ID, "taper": "XYZ" })).contains("unknown taper 'XYZ'"));
    assert!(m.refused("edit_user_holder", json!({ "id": ID, "name": "  " })).contains("needs a name"));
    assert!(m.refused("edit_user_holder", json!({ "id": ID, "gauge_len": 80.0 })).contains("already so"));
    assert!(m.refused("edit_user_holder", json!({ "id": ID, "colour": "red" })).contains("unknown argument \"colour\""));

    assert_eq!(row(&mut m), before_row, "no refusal changes the holder");
    assert_eq!(m.ok("get_tool_assembly", json!({ "tool": A })), before, "or the tool");
}
