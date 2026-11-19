//! The interface's own translation gates (#853): every word the ribbon, the
//! tree and the machine tabs can wear has a Chinese string, and every Chinese
//! ribbon caption fits the cell the theme gives it - the layout worry answered
//! with a measurement, not an opinion.

use super::*;
use crate::egui_app::fonts;
use camcraft_i18n::{missing, tr_in, Lang};

/// The ribbon's fixed cells: `Tokens::cell_w` and `cell_h` (theme/tables.rs).
const CELL_W: f32 = 76.0;
const LABEL_PX: f32 = 11.0;

fn every_ribbon_string() -> Vec<String> {
    let mut out = Vec::new();
    for c in [
        Cmd::New,
        Cmd::Model,
        Cmd::Project,
        Cmd::Save,
        Cmd::SaveAs,
        Cmd::Post,
        Cmd::Setups,
        Cmd::Stock,
        Cmd::Origin,
        Cmd::Tools,
        Cmd::Machine,
        Cmd::Undo,
        Cmd::Redo,
        Cmd::Run,
        Cmd::Delete,
        Cmd::Frame,
        Cmd::Paths,
        Cmd::Simulate,
        Cmd::Deviate,
        Cmd::View,
        Cmd::Theme,
    ] {
        out.push(label_of(c).to_owned());
    }
    for s in camcraft_app::operation::Section::ORDER {
        out.push(door_label(s).to_owned());
        out.push(s.label().to_owned());
    }
    for k in camcraft_app::operation::KindInfo::ALL {
        out.push(k.caption.to_owned());
    }
    for (_, caption, _) in crate::machine_modes::MACHINE_MODES {
        out.push((*caption).to_owned());
    }
    for g in ["File", "Setup", "Modify", "Inspect", "View", "Pinned"] {
        out.push(g.to_owned());
    }
    // The tree's literal rows (browser_tree's `t(...)` sites) and the tool
    // kind names its rows carry.
    for t in [
        "SETUP", "Setups", "MODELS", "(no models yet)", "TOOLS", "(no tools yet)", "WCS", "Stock",
        "Operations", "(no operations yet)", "Tool list", "Fixtures", "Machine", "Post", "not set",
        "active", " (fixture)", " (hidden)", "operation", "tool", "fixture",
    ] {
        out.push(t.to_owned());
    }
    for t in [
        "end mill", "ball nose", "bull nose", "drill", "tap", "boring bar", "turning insert",
        "grooving blade", "threading insert", "form gauge", "chamfer mill", "touch probe",
        "plasma torch", "waterjet nozzle", "sinker electrode", "thread mill",
        "single-form thread mill", "EDM wire",
    ] {
        out.push(t.to_owned());
    }
    out
}

#[test]
fn every_ribbon_and_tree_string_has_a_chinese_translation() {
    let missed: Vec<String> = every_ribbon_string()
        .into_iter()
        // `2D` / `3D` are written the same in Chinese; they fall back on purpose.
        .filter(|s| s != "2D" && s != "3D" && missing(Lang::ZhCn, s))
        .collect();
    assert!(missed.is_empty(), "no zh-CN string for {missed:?}");
}

#[test]
fn every_property_row_label_used_by_the_common_pages_has_a_translation() {
    // The page helpers route every row through `label_cell`, so this list is
    // the shared vocabulary the milling pages draw; a new row that forgets the
    // catalogue lands here.
    let rows = [
        "Feed", "Plunge feed", "Spindle speed", "Stepdown", "Stepover", "Tolerance",
        "Stock to leave", "Climb milling", "Strategy", "Side", "Depth", "Height above material",
        "Lead in/out", "Lead length", "Lead arc radius", "Ramp angle", "Max ramp angle",
        "Between levels", "Between passes", "Level to level", "Entry", "Linking", "Boundaries",
        "Containment", "Boundary offset", "Tool", "Compensation", "Spring passes", "Finish allowance",
        "Direction", "Cycle", "Dwell at bottom", "Thread pitch", "Pitch", "Flutes", "Material",
        "Work offset", "Machined side", "Output", "Program number", "Program comment",
        "Output folder", "Holes", "Minimum diameter", "Maximum diameter",
    ];
    let missed: Vec<&str> = rows.into_iter().filter(|s| missing(Lang::ZhCn, s)).collect();
    assert!(missed.is_empty(), "no zh-CN string for {missed:?}");
}

#[test]
fn every_chinese_ribbon_caption_fits_its_cell() {
    let ctx = egui::Context::default();
    fonts::install_for(&ctx, Lang::ZhCn);
    warm(&ctx);
    let mut too_wide = Vec::new();
    for source in every_ribbon_string() {
        let zh = tr_in(Lang::ZhCn, &source).into_owned();
        let galley = ctx.fonts_mut(|f| {
            f.layout_no_wrap(zh.clone(), egui::FontId::proportional(LABEL_PX), egui::Color32::WHITE)
        });
        // `painted::cell` reserves 2 px of the cell and shrinks a label that
        // still overflows, so this is the width that must be free.
        if galley.size().x > CELL_W - 2.0 {
            too_wide.push((source, zh, galley.size().x));
        }
    }
    assert!(too_wide.is_empty(), "labels wider than the cell: {too_wide:?}");
}

#[test]
fn the_chinese_ribbon_is_narrower_than_the_english_one() {
    // The premise the layout risk rests on: at the same size, the translated
    // captions measure no wider than the source ones on average.
    let ctx = egui::Context::default();
    fonts::install_for(&ctx, Lang::ZhCn);
    warm(&ctx);
    let (mut en, mut zh) = (0.0, 0.0);
    let strings = every_ribbon_string();
    for s in &strings {
        let e = ctx.fonts_mut(|f| f.layout_no_wrap(s.clone(), egui::FontId::proportional(LABEL_PX), egui::Color32::WHITE));
        let zh_text = tr_in(Lang::ZhCn, s).into_owned();
        let z = ctx.fonts_mut(|f| {
            f.layout_no_wrap(zh_text, egui::FontId::proportional(LABEL_PX), egui::Color32::WHITE)
        });
        en += e.size().x;
        zh += z.size().x;
    }
    assert!(zh < en, "zh {zh:.0} px vs en {en:.0} px over {} strings", strings.len());
}

/// One frame, so egui builds its font atlas; the delta is dropped on purpose
/// (a test owns no GPU texture to apply it to).
fn warm(ctx: &egui::Context) {
    let mut out = ctx.run_ui(egui::RawInput::default(), |_| {});
    out.textures_delta.clear();
}
