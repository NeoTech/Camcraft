//! Shared fixtures of the axis chain tests (#2756: `axis_chain`,
//! `axis_rows`): the shipped grblHAL mill with unknown keys, a
//! tilting head of our own, and readers of the parsed axes. Not a suite.
//!
//! Fixture law (clean-room): the shipped grblHAL mill plus our own
//! values in the `.mch` layout of `assets/machines/AGENTS.md`.

#![allow(dead_code, unused_imports)]

pub use std::path::PathBuf;

pub use camcraft_app::machine::chain::{Place, Step};
pub use camcraft_app::machine::draft::MachineDraft;
pub use camcraft_app::machine::library::MachineSource;
pub use camcraft_app::machine::validate::validate_text;
pub use camcraft_app::machine::{parse_mch, AxisLimits};
pub use serde_json::{json, Value};


/// The shipped 3-axis grblHAL mill: Z carries Y carries X on the
/// table, and the head sits beside Z.
pub fn grblhal() -> MachineDraft {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/machines/grblhal-3axis.mch");
    let mut m = MachineDraft::open(&MachineSource::File(p)).expect("opens");
    // Keys the editor knows nothing about, at the top and on an axis.
    m.value_mut()["x_vendor_block"] = json!({ "probe": "OMP40" });
    m.value_mut()["kinematics"]["default"]["parts"][0]["parts"][0]["parts"][0]["parts"][0]["note"] = json!("gib adjusted 2026");
    m
}

pub fn axes(m: &MachineDraft) -> Vec<AxisLimits> {
    parse_mch(&m.text()).expect("parses").axes
}

pub fn axis(m: &MachineDraft, name: &str) -> AxisLimits {
    axes(m).into_iter().find(|a| a.name == name).unwrap_or_else(|| panic!("no axis {name}"))
}

pub fn names(m: &MachineDraft) -> Vec<String> {
    axes(m).into_iter().map(|a| a.name).collect()
}

pub fn unknown_keys_kept(v: &Value) {
    assert_eq!(v.pointer("/x_vendor_block/probe"), Some(&json!("OMP40")));
    let x = v.pointer("/kinematics/default/parts/0/parts/0/parts/0/parts/0").expect("X");
    assert_eq!(x.get("name"), Some(&json!("X")));
    assert_eq!(x.get("note"), Some(&json!("gib adjusted 2026")));
}

/// A tilting head of our own: Z carries A carries B carries the head;
/// B states every #1962 key and a key the editor does not know.
pub fn tilting_head() -> MachineDraft {
    let mut m = grblhal();
    m.add_rotary_axis(Place::CarriesHead, "A", [1.0, 0.0, 0.0]).expect("adds A");
    let b = m.add_rotary_axis(Place::CarriesHead, "B", [0.0, 1.0, 0.0]).expect("adds B");
    let part = m.value_mut().pointer_mut(&b).expect("B");
    part["pivot"] = json!([0.0, 0.0, 150.0]);
    part["offset"] = json!([0.0, 0.0, -40.0]);
    part["zero_offset"] = json!(0.5);
    part["rollover"] = json!(true);
    part["sign_convention"] = json!("tool_rh");
    part["note"] = json!("trunnion B, serial 42");
    m
}

pub fn trunnion_keys(m: &MachineDraft, name: &str) -> Value {
    let part = m.value().pointer(&pointer_of(m, name)).expect("part").clone();
    let keep = ["id", "pivot", "offset", "zero_offset", "rollover", "sign_convention", "note"];
    keep.iter().filter_map(|k| part.get(*k).map(|x| ((*k).to_owned(), x.clone()))).collect::<serde_json::Map<_, _>>().into()
}

/// The pointer of the axis part named `name`, depth-first.
pub fn pointer_of(m: &MachineDraft, name: &str) -> String {
    fn walk(v: &Value, at: &str, name: &str) -> Option<String> {
        v.pointer(at)?.as_array()?.iter().enumerate().find_map(|(i, p)| {
            let here = format!("{at}/{i}");
            if p.get("name").and_then(Value::as_str) == Some(name) {
                Some(here)
            } else {
                walk(v, &format!("{here}/parts"), name)
            }
        })
    }
    walk(m.value(), "/kinematics/default/parts", name).unwrap_or_else(|| panic!("no part {name}"))
}


/// The editor field labelled `label` (panics when there is none).
pub fn field(m: &MachineDraft, label: &str) -> camcraft_app::machine::fields::Field {
    let f = m.groups().into_iter().flat_map(|g| g.fields).find(|f| f.label == label);
    f.unwrap_or_else(|| panic!("no field {label}"))
}
