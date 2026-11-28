//! #4095: G247 Q339 selects a preset-table row only where cycle 247 has
//! that meaning. Before iTNC 530 340 422 it sets a datum value, and the TNC
//! 410 has no cycle 247 (docs/research/heidenhain/differences.md B2, "Cycle
//! 247 DATUM SETTING"). The post refuses the block by name there.

use super::*;

fn preset_hole(model: Option<(&str, &str)>) -> engine::PostOutput {
    let mut rs = vec![param("WorkOffset", json!(2))];
    rs.extend(setup(3000.0));
    let mut n = 0;
    add_hole(&mut rs, &mut n, hole(CycleKind::Drill, 30.0, 0.0, -8.0, 0.0, 0.0, Some(90.0)), None);
    finish(&mut rs);
    let mut m = machine(None);
    if let Some((model, nc)) = model {
        m["controller"]["identity"] = json!({ "model": model, "nc_software": nc });
    }
    engine::run(&read("assets/posts/heidenhain.cps"), &m, &stream(&rs))
        .unwrap_or_else(|e| panic!("Heidenhain engine run: {e}"))
}

#[test]
fn a_preset_table_control_selects_the_row() {
    for id in [None, Some(("iTNC 530", "340422-01")), Some(("iTNC 530", "340490-03")), Some(("TNC 640", "340595-11"))] {
        let out = preset_hole(id);
        assert!(out.warnings.is_empty(), "{id:?}: {:?}", out.warnings);
        assert!(bodies(&out.program).iter().any(|b| b == "G247 Q339=2"), "{id:?}: {}", out.program);
    }
}

#[test]
fn g247_is_refused_where_cycle_247_is_not_a_preset_row() {
    for (model, nc, why) in [
        ("iTNC 530", "340420-05", "iTNC 530 NC 340420-05 (profile itnc-340420): its cycle 247 sets a datum value, not a preset-table row"),
        ("TNC 430", "280476-01", "TNC 430 NC 280476-01 (profile tnc426-476): its cycle 247 sets a datum value, not a preset-table row"),
        ("TNC 410", "286060-01", "TNC 410 NC 286060-01 (profile tnc410): it has no cycle 247"),
        ("TNC 426", "280474-01", "TNC 426 NC 280474-01 (profile tnc426-474): the meaning of its cycle 247 is not known"),
    ] {
        let out = preset_hole(Some((model, nc)));
        let want = format!("G247 Q339=2 (preset-table row) cannot be written for {why} - refused, not posted");
        assert!(out.warnings.iter().any(|w| w.contains(&want)), "{model} {nc}: {:?}", out.warnings);
        assert!(!out.program.contains("G247"), "{model} {nc}: {}", out.program);
    }
}
