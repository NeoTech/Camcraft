//! #3071: hobby router frames (Carbide 3D Shapeoko 5 Pro, Inventables
//! X-Carve and X-Carve Pro) - each machine file states only the figures
//! that its maker publishes, names the Grbl 1.1 post, and a two-tool
//! program posts through it: mm, an M0 stop for each tool change (Grbl
//! 1.1 has no M6), the Z lift in the work offset (no maker states where
//! machine Z0 sits, so no `safe_retract`), and a spindle speed past the
//! stated maximum refused.
//! Provenance: our own tests over our own machine files and post. The
//! figures are from the makers' public pages (Carbide 3D "Shapeoko 5.1
//! Pro Specifications", carbide3d.com; Inventables' machine pages on
//! easel.com; read 2026-11-26). See docs/decisions/3071-hobby-routers.md.

use camcraft_app::machine::{Machine, parse_mch};
use camcraft_app::post::{engine, machine};
use serde_json::Value;
use std::path::{Path, PathBuf};

/// Two tools, a cut and a plunge (Grbl 1.1 has no canned cycles, so the
/// hole is its plain moves): enough to show the lift, the change and the
/// program's shape.
const STREAM: &str = "PARAM ClearancePlane 15\nPARAM StockZTop 0\nPARAM SpindleSpeed 18000\nTOOL 3 EM6\nSECTION\nRapid X 0 Y 0 Z 5000\nFeed X 10000 Y 0 Z -1000 F 1500000\nENDSECTION\n\
                      PARAM ClearancePlane 15\nPARAM SpindleSpeed 12000\nTOOL 4 DRILL5\nSECTION\nRapid X 20000 Y 5000 Z 2000\nFeed X 20000 Y 5000 Z -3000 F 300000\nRapid X 20000 Y 5000 Z 2000\nENDSECTION\n";

fn repo(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../").join(rel)
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(repo(rel)).unwrap_or_else(|e| panic!("{rel}: {e}"))
}

fn shipped(name: &str) -> Machine {
    let text = camcraft_app::posting::builtin_machine_text(name)
        .unwrap_or_else(|why| panic!("{name} is not a shipped machine: {why}"));
    parse_mch(text).unwrap_or_else(|e| panic!("{name}: {e}"))
}

fn mirrored(name: &str) -> Value {
    machine::mirror(&shipped(name))
}

fn post(m: &Value, stream: &str) -> engine::PostOutput {
    engine::run(&read("assets/posts/grbl.cps"), m, stream).unwrap_or_else(|e| panic!("posts: {e}"))
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-6
}

/// One frame's published figures: travel in mm, and the spindle's rpm
/// range and kW when the maker names the spindle that ships with it.
struct Spec {
    name: &'static str,
    travel: [f64; 3],
    spindle: Option<(f64, f64, f64)>,
}

const SPECS: [Spec; 6] = [
    // Carbide 3D's spec table: travel (1249 - 12) mm on a 4 ft axis and
    // (635 - 12) mm on a 2 ft axis; total Z travel (165 - 10) = 155 mm.
    // The router is the owner's choice, so no spindle is stated.
    Spec { name: "Carbide 3D Shapeoko 5 Pro 4x4", travel: [1237.0, 1237.0, 155.0], spindle: None },
    Spec { name: "Carbide 3D Shapeoko 5 Pro 4x2", travel: [1237.0, 623.0, 155.0], spindle: None },
    Spec { name: "Carbide 3D Shapeoko 5 Pro 2x2", travel: [623.0, 623.0, 155.0], spindle: None },
    // Inventables' 2021 X-Carve page: 29.5 x 29.5 x 4.5 in cuttable area;
    // the Makita router, 1.25 HP (800 W), 10,000 to 30,000 rpm.
    Spec { name: "Inventables X-Carve 2021 1000 mm", travel: [749.3, 749.3, 114.3], spindle: Some((10000.0, 30000.0, 0.8)) },
    // Inventables' X-Carve Pro page: 48 x 48 x 4 in and 48 x 24 x 4 in;
    // the 1.5 kW VFD spindle, 8,000 to 24,000 rpm.
    Spec { name: "Inventables X-Carve Pro 4x4", travel: [1219.2, 1219.2, 101.6], spindle: Some((8000.0, 24000.0, 1.5)) },
    Spec { name: "Inventables X-Carve Pro 4x2", travel: [1219.2, 609.6, 101.6], spindle: Some((8000.0, 24000.0, 1.5)) },
];

#[test]
fn every_hobby_router_states_only_its_published_figures() {
    for s in &SPECS {
        let n = s.name;
        let m = shipped(n);
        assert_eq!(m.model, n);
        assert_eq!(m.units, None, "{n}: mm (the parser records only inch)");
        assert_eq!(m.axes.len(), 3, "{n}: X, Y, Z");
        for (a, want) in m.axes.iter().zip(s.travel) {
            // The full stroke either way from the work zero, as every
            // shipped machine writes it.
            let (lo, hi) = a.travel.unwrap_or_else(|| panic!("{n} {} travel", a.name));
            assert!(close(hi, want) && close(lo, -want), "{n} {}: {lo}..{hi}", a.name);
            assert_eq!((a.feed, a.rapid), (None, None), "{n} {}: no maker states the axis rates", a.name);
        }
        let spindle = match (m.spindle_min, m.spindle_max, m.spindle_power) {
            (Some(lo), Some(hi), Some(kw)) => Some((lo, hi, kw)),
            (None, None, None) => None,
            other => panic!("{n}: a partial spindle {other:?}"),
        };
        assert_eq!(spindle, s.spindle, "{n} spindle");
        assert_eq!(m.safe_retract_z, None, "{n}: no maker states where machine Z0 sits");
        assert!(m.coolants.is_empty(), "{n}: no coolant is stated");
    }
}

#[test]
fn every_tool_change_is_a_stop_after_a_lift_in_the_work_offset() {
    for s in &SPECS {
        let out = post(&mirrored(s.name), STREAM);
        assert!(out.warnings.is_empty(), "{}: {:?}", s.name, out.warnings);
        let nc = out.program;
        let lines: Vec<&str> = nc.lines().collect();
        assert_eq!(lines.first().copied(), Some(format!("({})", s.name).as_str()), "{nc}");
        assert!(nc.lines().any(|l| l.starts_with("G21 ")), "mm\n{nc}");
        let stop = lines.iter().position(|l| *l == "M0").unwrap_or_else(|| panic!("{}: no M0 stop\n{nc}", s.name));
        assert!(lines[..stop].iter().rev().take(3).any(|l| l.contains("LOAD T4")), "{}: the stop names T4\n{nc}", s.name);
        assert!(lines[..stop].contains(&"G0 Z15"), "{}: Z goes up first\n{nc}", s.name);
        assert_eq!(nc.matches("\nM0\n").count(), 1, "one change between two tools\n{nc}");
        for w in nc.split_whitespace() {
            assert!(!["M6", "G53", "G28", "G30"].contains(&w), "{}: {w} is not written\n{nc}", s.name);
        }
    }
}

#[test]
fn a_spindle_speed_past_the_stated_maximum_is_refused() {
    let stream = STREAM.replacen("PARAM SpindleSpeed 18000", "PARAM SpindleSpeed 26000", 1);
    let out = post(&mirrored("Inventables X-Carve Pro 4x4"), &stream);
    assert!(out.warnings.iter().any(|w| w.contains("spindle rpm over machine maximum")), "{:?}", out.warnings);
    // The 2021 X-Carve's router turns to 30,000 rpm: the same program posts.
    let out = post(&mirrored("Inventables X-Carve 2021 1000 mm"), &stream);
    assert!(out.warnings.is_empty(), "{:?}", out.warnings);
}

/// The two-tool program on the Shapeoko 5 Pro 4x4, locked byte-exact, so
/// a later post change shows up as a diff.
#[test]
fn the_shapeoko_4x4_two_tool_program_is_locked() {
    let out = post(&mirrored("Carbide 3D Shapeoko 5 Pro 4x4"), STREAM);
    assert!(out.warnings.is_empty(), "{:?}", out.warnings);
    let rel = "crates/app/tests/golden/nc/grbl/shapeoko_5_pro_4x4_two_tool.nc";
    if std::env::var_os("WRITE_GOLDEN").is_some() {
        std::fs::write(repo(rel), &out.program).expect("golden written");
    }
    assert_eq!(out.program, read(rel), "{rel}: run with WRITE_GOLDEN=1 and review the diff");
}
