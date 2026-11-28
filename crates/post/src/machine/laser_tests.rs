//! #4081: a laser machine file is an X/Y gantry with an optional focus Z,
//! no spindle and no rotary axis; each broken door rule is refused by its
//! key, in the file form and in the saved-project form.
//! Provenance: our own synthetic machine fixture and the issue's rules.

use crate::machine::{kind, laser_refusals, parse_mch, Coolant, Kind, Machine};
use serde_json::{json, Value};

fn fixture() -> Value {
    json!({
        "general": { "model": "Laser test", "kind": "laser" },
        "kinematics": { "default": {
            "units": { "length": "mm", "angle": "degrees" },
            "parts": [
                { "id": "x", "name": "X", "type": "linear", "travel": { "min": 0.0, "max": 400.0 } },
                { "id": "y", "name": "Y", "type": "linear", "travel": { "min": 0.0, "max": 300.0 } },
                { "id": "beam", "name": "laser head", "type": "head" }
            ]
        } }
    })
}

#[test]
fn a_laser_word_reads_as_the_laser_kind_and_round_trips() {
    for w in ["laser", "Laser", "laser cutter", "LASER-CUTTING"] {
        assert_eq!(Kind::from_word(w), Some(Kind::Laser), "{w}");
    }
    assert_eq!(Kind::from_word(Kind::Laser.word()), Some(Kind::Laser));
    assert_eq!(Kind::Laser.label(), "laser cutting");
}

#[test]
fn a_laser_without_z_parses_and_an_optional_focus_z_is_accepted() {
    let m = parse_mch(&fixture().to_string()).expect("an X/Y laser needs no Z axis");
    assert_eq!(kind(Some(&m)), Kind::Laser);
    assert!(m.is_sane());
    assert_eq!(m.axis(0).and_then(|a| a.travel), Some((0.0, 400.0)));
    assert!(m.axis(2).is_none());
    assert!(m.summary().contains("laser head"), "{}", m.summary());

    let mut v = fixture();
    let parts = v["kinematics"]["default"]["parts"].take();
    v["kinematics"]["default"]["parts"] = json!([
        { "name": "Z", "type": "linear", "travel": { "min": 0.0, "max": 50.0 }, "parts": parts }
    ]);
    assert!(laser_refusals(&v).is_empty());
    let m = parse_mch(&v.to_string()).expect("a focus Z is optional, not forbidden");
    assert!(m.is_sane());
    assert_eq!(m.axis(2).and_then(|a| a.travel), Some((0.0, 50.0)));
}

#[test]
fn a_laser_file_refuses_each_forbidden_key_by_name() {
    for (key, head) in [
        ("spindle.max_speed", json!({"spindle": {"max_speed": 1000.0}})),
        ("live_tool", json!({"tool_station": {"live_tool": {}}})),
        ("coolants", json!({"tool_station": {"coolants": ["Flood"]}})),
    ] {
        let mut v = fixture();
        let head_part = v["kinematics"]["default"]["parts"][2].as_object_mut().expect("the head part");
        head_part.extend(head.as_object().expect("an object").clone());
        assert!(laser_refusals(&v).iter().any(|(k, _)| k == key), "{key}");
        let error = parse_mch(&v.to_string()).expect_err(key);
        assert!(error.contains(key) && error.contains("a laser"), "{error}");
    }
    let mut v = fixture();
    v["kinematics"]["default"]["parts"].as_array_mut().expect("parts").push(json!({"name": "A", "type": "rotary"}));
    let error = parse_mch(&v.to_string()).expect_err("a rotary");
    assert!(error.contains("A: a laser states no rotary axis"), "{error}");
    for (index, name) in [(0, "X"), (1, "Y")] {
        let mut v = fixture();
        v["kinematics"]["default"]["parts"].as_array_mut().expect("parts").remove(index);
        let error = parse_mch(&v.to_string()).expect_err("a missing axis");
        assert!(error.contains(&format!("{name} axis: required for a laser machine")), "{error}");
    }
}

#[test]
fn a_saved_laser_refuses_a_spindle_a_coolant_or_a_rotary() {
    let base = parse_mch(&fixture().to_string()).expect("the fixture parses");
    type Mutation = (&'static str, fn(&mut Machine));
    let mutations: &[Mutation] = &[
        ("spindle_max", |m| m.spindle_max = Some(1000.0)),
        ("spindle_power", |m| m.spindle_power = Some(1.0)),
        ("coolants", |m| m.coolants.push(Coolant { name: "Flood".into(), m_code: "M8".into() })),
        ("rotary", |m| { let mut a = m.axes[0].clone(); a.name = "A".into(); a.rotary = true; m.axes.push(a); }),
        ("missing Y", |m| m.axes.retain(|a| a.name != "Y")),
    ];
    for (name, mutate) in mutations {
        let mut m = base.clone();
        mutate(&mut m);
        assert!(!m.laser_sane() && !m.is_sane(), "{name}");
    }
    let mill = Machine { kind: Some("mill".into()), spindle_max: Some(1000.0), ..base };
    assert!(mill.laser_sane(), "the laser rules leave a mill alone");
}

#[test]
fn a_laser_keeps_its_kind_through_the_post_mirror() {
    let m = parse_mch(&fixture().to_string()).expect("the fixture parses");
    let mirrored = crate::post::machine::mirror(&m);
    assert_eq!(mirrored["kind"], "laser", "{mirrored}");
}
