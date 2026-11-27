//! The HEIDENHAIN profile table (#4095): control model, NC software family
//! and release range to the flag values of the profile.
//!
//! Provenance: `docs/research/heidenhain/dialects.md` section 3 (profiles)
//! and `differences.md` sections B, B2 and B3 (HEIDENHAIN manual IDs and
//! pages). Each flag and its source is listed in
//! `docs/decisions/4095-heidenhain-dialect-resolver.md`. Release lists name
//! only the releases that the research names (as #4172 did): an unnamed
//! release is refused, never assumed.

use serde::Serialize;

/// A fact that the research settles (`Yes`/`No`) or does not (`Unknown`).
/// A post refuses a function whose flag is `Unknown`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Tri {
    Yes,
    No,
    Unknown,
}

/// The cycle 200 (DRILLING) parameter level (`differences.md` B2, "Cycle
/// 200 Q list"). Each level includes the one before.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum C200 {
    /// Q200 Q201 Q206 Q202 Q210 Q203 Q204.
    Base,
    /// + Q211 (dwell time at the depth).
    Q211,
    /// + Q395 (depth reference).
    Q395,
}

/// What cycle 247 (DATUM SETTING) does (`differences.md` B2, "Cycle 247").
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Cycle247 {
    /// The control has no cycle 247.
    None,
    /// It sets a datum value (before iTNC 530 340 422).
    Datum,
    /// It activates a row of the preset table.
    PresetTable,
    Unknown,
}

/// The Klartext dwell form (`differences.md` B2, "Cycle 9" and
/// "FUNCTION DWELL"). DIN/ISO always writes `G04 F` (cycle 9).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Dwell {
    Cycle9,
    FunctionDwell,
    Unknown,
}

/// The capability flags of one profile.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Flags {
    /// DIN/ISO programs exist on this control and version.
    pub language_iso: Tri,
    /// Coordinate decimals: 3 (1 µm input) or 4 (0.1 µm input).
    pub decimals: u8,
    /// `TOOL CALL "NAME"` is permitted.
    pub tool_call_by_name: Tri,
    /// The tool name length in TOOL CALL / TOOL DEF (0 without names).
    pub tool_name_max: u8,
    /// `L M140 MB MAX` (retract along the tool axis).
    pub m140: Tri,
    /// M136/M137 (feed per revolution / per minute).
    #[serde(rename = "m136m137")]
    pub m136_m137: Tri,
    /// The cycle 200 (DRILLING) parameter level.
    pub drill_c200: C200,
    /// The meaning of cycle 247 (DATUM SETTING).
    pub cycle247: Cycle247,
    /// The Klartext dwell form.
    pub dwell: Dwell,
}

/// The NC software identifiers of one row.
pub(super) enum Ids {
    /// Named 6-digit identifiers (the old controls, iTNC 530 340 420/422).
    Exact(&'static [u32]),
    /// A 5-digit family and a variant range (the sixth digit).
    Family { fam: u32, vmin: u32, vmax: u32 },
}

/// The releases of one row.
pub(super) enum Rels {
    /// The research names no release: each release `01`..`99`.
    Any,
    Range(u8, u8),
    List(&'static [u8]),
}

pub(super) struct Row {
    pub profile: &'static str,
    pub models: &'static [&'static str],
    pub ids: Ids,
    pub rels: Rels,
    pub flags: Flags,
}

/// TNC 124 and TNC 128: one axis in each block, no arcs (`differences.md` A.1).
pub(super) const PARAXIAL: &[&str] = &["TNC 124", "TNC 128"];

use C200::{Base, Q211, Q395};
use Tri::{No, Unknown, Yes};

const TNC310: Flags = Flags {
    language_iso: No,
    decimals: 3,
    tool_call_by_name: No,
    tool_name_max: 0,
    m140: No,
    m136_m137: Unknown,
    drill_c200: Base,
    cycle247: Cycle247::None,
    dwell: Dwell::Unknown,
};
const TNC410: Flags = Flags { language_iso: Yes, m136_m137: No, dwell: Dwell::Cycle9, ..TNC310 };
const TNC426A: Flags = Flags {
    language_iso: Unknown,
    decimals: 4,
    tool_call_by_name: Yes,
    tool_name_max: 16,
    m140: No,
    m136_m137: Unknown,
    drill_c200: Base,
    cycle247: Cycle247::Unknown,
    dwell: Dwell::Unknown,
};
const TNC426_472: Flags = Flags { language_iso: Yes, dwell: Dwell::Cycle9, ..TNC426A };
const TNC426_474: Flags = Flags { drill_c200: Q211, ..TNC426_472 };
const TNC426_476: Flags = Flags { m140: Yes, cycle247: Cycle247::Datum, ..TNC426_474 };
const ITNC_420: Flags = Flags {
    language_iso: Yes,
    decimals: 4,
    tool_call_by_name: Yes,
    tool_name_max: 16,
    m140: Yes,
    m136_m137: Yes,
    drill_c200: Q211,
    cycle247: Cycle247::Datum,
    dwell: Dwell::Cycle9,
};
const ITNC_422: Flags = Flags { cycle247: Cycle247::PresetTable, ..ITNC_420 };
const ITNC_604: Flags = Flags { drill_c200: Q395, ..ITNC_422 };
const TNC320_EARLY: Flags = Flags {
    language_iso: Unknown,
    decimals: 4,
    tool_call_by_name: Yes,
    tool_name_max: 16,
    m140: Yes,
    m136_m137: Unknown,
    drill_c200: Q211,
    cycle247: Cycle247::Unknown,
    dwell: Dwell::Cycle9,
};
const TNC320_03: Flags = Flags { cycle247: Cycle247::PresetTable, ..TNC320_EARLY };
const TNC320_04: Flags = Flags { language_iso: Yes, ..TNC320_03 };
const TNC320_77_OLD: Flags = Flags { drill_c200: Q395, ..TNC320_04 };
const TNC320_77: Flags = Flags { dwell: Dwell::FunctionDwell, ..TNC320_77_OLD };
const TNC620_EARLY: Flags = Flags { cycle247: Cycle247::PresetTable, ..TNC320_EARLY };
const TNC620_MID: Flags = Flags { language_iso: Yes, ..TNC620_EARLY };
const TNC620_817_OLD: Flags = Flags { drill_c200: Q395, ..TNC620_MID };
const TNC620_817: Flags = Flags { dwell: Dwell::FunctionDwell, ..TNC620_817_OLD };
const TNC640_01: Flags = TNC620_MID;
const TNC640_02: Flags = Flags { tool_name_max: 32, ..TNC640_01 };
const TNC640_B: Flags = Flags { drill_c200: Q395, ..TNC640_02 };
const TNC640_C: Flags = Flags { dwell: Dwell::FunctionDwell, ..TNC640_B };
const TNC7_16: Flags = Flags { language_iso: No, ..TNC640_C };
const TNC7: Flags = Flags { language_iso: Yes, ..TNC640_C };

const TNC426: &[&str] = &["TNC 426", "TNC 430"];
const ITNC: &[&str] = &["iTNC 530"];
const fn fam(fam: u32, vmin: u32, vmax: u32) -> Ids {
    Ids::Family { fam, vmin, vmax }
}
const fn row(profile: &'static str, models: &'static [&'static str], ids: Ids, rels: Rels, flags: Flags) -> Row {
    Row { profile, models, ids, rels, flags }
}

/// The variant ranges follow #4172: iTNC 530 34049 + 0..4 and 60642 + 0..5
/// (`dialects.md` section 1); the TNC 320/620/640/TNC7 families take 0..5.
pub(super) const ROWS: &[Row] = &[
    row("tnc310", &["TNC 310"], Ids::Exact(&[286040, 286140]), Rels::Any, TNC310),
    row("tnc410", &["TNC 410"], Ids::Exact(&[286060, 286080]), Rels::Any, TNC410),
    row("tnc426a", &["TNC 426"], Ids::Exact(&[280462, 280470]), Rels::Any, TNC426A),
    row("tnc426-472", TNC426, Ids::Exact(&[280472]), Rels::Any, TNC426_472),
    row("tnc426-474", TNC426, Ids::Exact(&[280474, 280475]), Rels::Any, TNC426_474),
    row("tnc426-476", TNC426, Ids::Exact(&[280476, 280477]), Rels::Any, TNC426_476),
    row("itnc-340420", ITNC, Ids::Exact(&[340420]), Rels::Any, ITNC_420),
    row("itnc-340422", ITNC, Ids::Exact(&[340422]), Rels::Any, ITNC_422),
    row("itnc-49x", ITNC, fam(34049, 0, 4), Rels::Range(1, 8), ITNC_422),
    row("itnc-49x", ITNC, fam(60642, 0, 5), Rels::Range(1, 3), ITNC_422),
    row("itnc-60642x-04", ITNC, fam(60642, 0, 5), Rels::List(&[4]), ITNC_604),
    row("tnc320-early", &["TNC 320"], fam(34055, 0, 5), Rels::Range(1, 2), TNC320_EARLY),
    row("tnc320-34055x", &["TNC 320"], fam(34055, 0, 5), Rels::List(&[3]), TNC320_03),
    row("tnc320-34055x", &["TNC 320"], fam(34055, 0, 5), Rels::Range(4, 6), TNC320_04),
    row("tnc320-77185x", &["TNC 320"], fam(77185, 0, 5), Rels::Range(1, 2), TNC320_77_OLD),
    row("tnc320-77185x", &["TNC 320"], fam(77185, 0, 5), Rels::List(&[4, 5, 6, 18]), TNC320_77),
    row("tnc620-early", &["TNC 620"], fam(34056, 0, 5), Rels::List(&[1]), TNC620_EARLY),
    row("tnc620-mid", &["TNC 620"], fam(34056, 0, 5), Rels::Range(2, 4), TNC620_MID),
    row("tnc620-mid", &["TNC 620"], fam(73498, 0, 5), Rels::Range(1, 2), TNC620_MID),
    row("tnc620-81760x", &["TNC 620"], fam(81760, 0, 5), Rels::Range(1, 3), TNC620_817_OLD),
    row("tnc620-81760x", &["TNC 620"], fam(81760, 0, 5), Rels::List(&[4, 5, 6, 7, 8, 16, 17, 18]), TNC620_817),
    row("tnc640-a", &["TNC 640"], fam(34059, 0, 5), Rels::List(&[1]), TNC640_01),
    row("tnc640-a", &["TNC 640"], fam(34059, 0, 5), Rels::List(&[2]), TNC640_02),
    row("tnc640-b", &["TNC 640"], fam(34059, 0, 5), Rels::Range(4, 6), TNC640_B),
    row("tnc640-c", &["TNC 640"], fam(34059, 0, 5), Rels::Range(7, 9), TNC640_C),
    row("tnc640-d", &["TNC 640"], fam(34059, 0, 5), Rels::List(&[10, 11, 16, 17, 18]), TNC640_C),
    row("tnc7", &["TNC7", "TNC7 basic"], fam(81762, 0, 5), Rels::List(&[16]), TNC7_16),
    row("tnc7", &["TNC7", "TNC7 basic"], fam(81762, 0, 5), Rels::Range(17, 20), TNC7),
];
