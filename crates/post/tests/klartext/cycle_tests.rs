//! #4227: the Klartext post writes cycles 200, 206 and 207 and the cycle 9
//! dwell for the iTNC 530 NC 340490-03 (Cycle Programming manual, NC software
//! 340 49x). PECK and DEEP post the checked chain as L blocks (#4224). Each
//! native cycle replaces only the hole chain that it does itself.

use super::*;

const CYCLES: &str = include_str!("cycles.cnc");

fn cycles(stream: &str) -> Result<String, String> {
    post(MCH, stream, &Value::Null)
}

/// Replace one exact text of the stream; the variant must differ.
fn variant(from: &str, to: &str) -> String {
    let out = CYCLES.replacen(from, to, 1);
    assert_ne!(out, CYCLES, "the variant must change the stream: {from}");
    out
}

/// The blocks of `nc` without their block numbers (continuation lines stay).
fn blocks(nc: &str) -> Vec<String> {
    nc.lines()
        .map(|l| match l.split_once(' ') {
            Some((n, rest)) if n.parse::<u32>().is_ok() => rest.to_string(),
            _ => l.trim().to_string(),
        })
        .collect()
}

/// The text of one operation, from its comment to the next one.
fn operation<'a>(nc: &'a str, name: &str) -> &'a str {
    let start = nc.find(&format!("; OPERATION {name}")).unwrap_or_else(|| panic!("{name}: {nc}"));
    let rest = &nc[start + 1..];
    let end = rest.find("; OPERATION ").map_or(nc.len(), |e| start + 1 + e);
    &nc[start..end]
}

#[test]
fn klartext_cycles_post_on_the_picomax_golden() {
    let nc = cycles(CYCLES).expect("the cycle stream posts");
    recorded::compare(SCRIPT, "picomax_cycles", "nc", &nc);
}

#[test]
fn drill_posts_cycle_200_with_the_whole_depth_and_the_dwell_at_depth() {
    let nc = cycles(CYCLES).unwrap();
    let drill = blocks(operation(&nc, "Drill"));
    let def = drill.iter().position(|b| b == "CYCL DEF 200 DRILLING ~").expect("cycle 200");
    assert_eq!(
        drill[def + 1..def + 9],
        [
            "Q200=+1 ~", "Q201=-10 ~", "Q206=+100 ~", "Q202=+10 ~", "Q210=+0 ~", "Q203=+5 ~",
            "Q204=+1 ~", "Q211=+0.5",
        ]
    );
    // One definition, one call at each hole, each at R over its hole.
    assert_eq!(drill.iter().filter(|b| b.starts_with("CYCL DEF")).count(), 1, "{nc}");
    let calls: Vec<_> = drill.iter().enumerate().filter(|(_, b)| *b == "CYCL CALL").map(|(i, _)| i).collect();
    assert_eq!(calls.len(), 2, "{nc}");
    assert_eq!(drill[calls[1] - 1], "L X+15 Y+5 Z+6 R0 FMAX", "{nc}");
    // The chain that the cycle does is not written again.
    assert!(!drill.iter().any(|b| b.starts_with("L ") && !b.ends_with("FMAX")), "{nc}");
    assert!(!drill.iter().any(|b| b.contains("DWELL TIME")), "{nc}");
}

#[test]
fn peck_and_deep_post_the_checked_chain_and_deep_dwells_with_cycle_9() {
    let nc = cycles(CYCLES).unwrap();
    assert!(!nc.contains("CYCL DEF 205") && !nc.contains("CYCL DEF 203"), "{nc}");
    let peck = blocks(operation(&nc, "Peck"));
    let cuts: Vec<_> = peck.iter().filter(|b| b.ends_with("R0 F100")).cloned().collect();
    assert_eq!(cuts, ["L X+5 Y-5 Z+2 R0 F100", "L X+5 Y-5 Z-2 R0 F100", "L X+5 Y-5 Z-5 R0 F100"]);
    assert!(peck.iter().any(|b| b == "L X+5 Y-5 Z+3 R0 FMAX"), "the chip-break hop: {nc}");
    assert!(!peck.iter().any(|b| b.starts_with("CYCL")), "{nc}");
    let deep = blocks(operation(&nc, "Deep"));
    let bottom = deep.iter().position(|b| b == "L X+15 Y-5 Z-5 R0 F100").expect("the last feed");
    assert_eq!(deep[bottom + 1..bottom + 3], ["CYCL DEF 9.0 DWELL TIME", "CYCL DEF 9.1 DWELL 0.3"]);
    assert_eq!(deep.iter().filter(|b| b.starts_with("CYCL DEF 9.0")).count(), 1, "{nc}");
}

#[test]
fn rigid_and_floating_taps_post_cycles_207_and_206() {
    let nc = cycles(CYCLES).unwrap();
    let rigid = blocks(operation(&nc, "Rigid tap"));
    let def = rigid.iter().position(|b| b == "CYCL DEF 207 RIGID TAPPING NEW ~").expect("cycle 207");
    assert_eq!(rigid[def + 1..def + 6], ["Q200=+1 ~", "Q201=-8 ~", "Q239=+1 ~", "Q203=+5 ~", "Q204=+1"]);
    assert_eq!(rigid[def + 6], "CYCL CALL");
    let floating = blocks(operation(&nc, "Floating tap"));
    let def = floating.iter().position(|b| b == "CYCL DEF 206 TAPPING NEW ~").expect("cycle 206");
    assert_eq!(
        floating[def + 1..def + 7],
        ["Q200=+1 ~", "Q201=-8 ~", "Q206=+500 ~", "Q211=+0.2 ~", "Q203=+5 ~", "Q204=+1"]
    );
    for op in [&rigid, &floating] {
        assert!(!op.iter().any(|b| b.starts_with("L ") && !b.ends_with("FMAX")), "{nc}");
    }
}

#[test]
fn each_klartext_cycle_refusal_names_its_cause() {
    let bore = variant("CYCLE DRILL X 105000 Y 30000", "CYCLE BORE X 105000 Y 30000");
    let off_r = variant(
        "Rapid X 105000 Y 30000 Z 11000\nCYCLE DRILL",
        "Rapid X 105000 Y 30000 Z 12000\nCYCLE DRILL",
    );
    let extra_peck = variant(
        "Plunge X 105000 Y 30000 Z 0 F 100000\nRapid X 105000 Y 30000 Z 11000",
        "Plunge X 105000 Y 30000 Z 5000 F 100000\nRapid X 105000 Y 30000 Z 6000\nPlunge X 105000 Y 30000 Z 0 F 100000\nRapid X 105000 Y 30000 Z 11000",
    );
    let tap_by_drill = variant("PARAM ToolKind tap\n", "PARAM ToolKind drill\n");
    let rigid_dwell = variant("CYCLE TAP X 105000 Y 30000 Z 2000 R 11000 ENTRY 10000 Q 0 P 0", "CYCLE TAP X 105000 Y 30000 Z 2000 R 11000 ENTRY 10000 Q 0 P 100");
    let floating_dwell = variant("Q 0 P 200 F 500000", "Q 0 P 600 F 500000");
    let lateral = variant(
        "Rapid X 105000 Y 20000 Z 15000\nENDSECTION",
        "Feed X 106000 Y 20000 Z 15000 F 100000\nENDSECTION",
    );
    let no_bottom = variant(
        "Plunge X 115000 Y 20000 Z 0 F 100000\nRapid X 115000 Y 20000 Z 12000",
        "Plunge X 115000 Y 20000 Z 500 F 100000\nRapid X 115000 Y 20000 Z 12000",
    );
    for (stream, diagnostic) in [
        (bore, "drilling cycle BORE: unsupported cycle kind"),
        (off_r, "the tool is not at the hole's R plane when the cycle starts"),
        (extra_peck, "the hole chain is not the motion of cycle 200"),
        (tap_by_drill, "a tapping cycle requires a tap tool"),
        (rigid_dwell, "cycle 207 has no dwell parameter"),
        (floating_dwell, "cycle 206 dwell must be at most 0.5 s"),
        (lateral, "a drill tool cuts only inside its drilling cycle (feed)"),
        (no_bottom, "the depth disagrees with the hole chain bottom"),
    ] {
        let why = cycles(&stream).expect_err(diagnostic);
        assert!(why.contains(diagnostic), "{diagnostic}: {why}");
    }
}

#[test]
fn rigid_tapping_is_refused_on_a_machine_without_it() {
    let mut machine: Value = serde_json::from_str(MCH).unwrap();
    machine["kinematics"]["default"]["parts"][1]["parts"][0]["spindle"]["rigid_tapping"] = Value::Bool(false);
    let why = post(&machine.to_string(), CYCLES, &Value::Null).expect_err("no rigid tapping");
    assert!(why.contains("rigid tapping is unavailable on this machine"), "{why}");
}
