//! #4336 (U10 of #3663): the Tool tab of the tool library window reads as
//! the mock-up. Each new field lands through the footer's Apply
//! (`ChangeTool`, the number through `SetToolNumber`) and undoes; the
//! stick-out changes the derived gauge length; a shank outside the
//! holder's clamping range is refused by the holder's name; "Open holder"
//! shows the holder view and keeps the draft; the side drawing is the
//! default view and a still frame builds no model.
//! Gate map: covers ui::egui_app::tool_library_dialog, ui::egui_app::tool_holder_picker, ui::egui_app::tool_settings_grid, ui::state.
//! Provenance: our own structural tests over the AccessKit tree and the
//! command history; no HSMWorks material involved.

use camcraft_app::holder_library;
use camcraft_app::tools::ToolMaterial;
use camcraft_app::units::Unit;

use super::tool_library_assembly::{holder, pick, shows, tall_window};
use super::tool_library_editor::{click_window_button, draft, library_window, tool};
use super::*;
use crate::egui_app::tool_library_dialog::assembly_group::{self, FITS, OPEN_HOLDER, REFUSED};
use crate::egui_app::tool_library_dialog::{model as preview_model, side_view, tool_tab};
use crate::state::{LibraryPage, PreviewMode};

/// A 6 mm end mill on an 8 mm shank (flutes 15, shank 40: stick-out 55).
const TOOL: &str = "EM-0600-4FL";

fn painted(h: &super::tool_library_editor::H) -> Vec<String> {
    super::structural::painted(h).into_iter().map(|(t, _)| t).collect()
}

/// The new Tool tab fields - flutes, helix angle, tool material, centre
/// cutting - are a draft until Apply; Apply lands them as one
/// `ChangeTool` and one undo gives the old tool back. The tool number
/// lands as its own `SetToolNumber`, and undoes.
#[test]
fn each_tool_tab_field_lands_through_apply_and_undoes() {
    let mut h = tall_window(TOOL);
    let win = library_window(&h);
    for label in [tool_tab::NAME, "Type", "Tool number", tool_tab::CUTTER, "Flutes", "Helix angle", "Tool material", tool_tab::SHANK] {
        assert!(win.query_all_by_label(label).next().is_some(), "the Tool tab has {label:?}");
    }
    let before = tool(&h, TOOL);
    let done = h.state().state.history.done_len();
    draft(&mut h).flutes = Some(3);
    draft(&mut h).helix_deg = "35".into();
    draft(&mut h).tool_material = Some(ToolMaterial::Carbide);
    draft(&mut h).center_cutting = !before.center_cutting;
    h.run_steps(2);
    assert_eq!(tool(&h, TOOL), before, "a draft does not write the document");
    click_window_button(&mut h, "Apply");
    let after = tool(&h, TOOL);
    assert_eq!(after.flutes, Some(3), "Apply lands the flutes: {}", h.state().state.status);
    assert_eq!(after.catalog.helix_deg, Some(35.0), "Apply lands the helix angle");
    assert_eq!(after.tool_material, Some(ToolMaterial::Carbide), "Apply lands the tool material");
    assert_eq!(after.center_cutting, !before.center_cutting, "Apply lands centre cutting");
    assert_eq!(h.state().state.history.done_len(), done + 1, "one Apply is one undo step");
    crate::actions::undo(&mut h.state_mut().state);
    h.run_steps(2);
    assert_eq!(tool(&h, TOOL), before, "one undo restores the old tool");
    // The tool number: its own command, from the same Apply.
    let st = &h.state().state;
    let have = model::find_by_name(&st.world, TOOL).and_then(|e| model::tool_number_of(&st.world, &e));
    let want = camcraft_app::tool_numbers::MAX;
    draft(&mut h).number = want;
    click_window_button(&mut h, "Apply");
    let st = &h.state().state;
    let now = model::find_by_name(&st.world, TOOL).and_then(|e| model::tool_number_of(&st.world, &e));
    assert_eq!(now, Some(want), "Apply lands the tool number: {}", st.status);
    crate::actions::undo(&mut h.state_mut().state);
    h.run_steps(2);
    let st = &h.state().state;
    let back = model::find_by_name(&st.world, TOOL).and_then(|e| model::tool_number_of(&st.world, &e));
    assert_eq!(back, have, "one undo gives the old tool number back");
}

/// The stick-out is typed; the gauge length is derived from it and the
/// holder's own gauge length, with the sum shown under it.
#[test]
fn the_stick_out_changes_the_gauge_length() {
    let mut h = tall_window(TOOL);
    pick(&mut h, "BT30 ER20");
    assert!(shows(&h, "Stick-out") && shows(&h, "Holder gauge length"), "the group has the two typed lengths");
    assert!(shows(&h, "115 mm"), "60 mm of the holder + 55 mm stick-out = 115 mm: {:?}", painted(&h));
    holder(&mut h).projection = 70.0;
    h.run_steps(2);
    assert!(shows(&h, "130 mm"), "60 mm of the holder + 70 mm stick-out = 130 mm");
    assert!(shows(&h, "= 60 mm of the holder + 70 mm stick-out."), "the sum is shown");
    let dims = preview_model::last().unwrap_or_else(|| panic!("the preview built a model")).dims;
    assert!((dims.gauge - 130.0).abs() < 1e-6, "the drawing's gauge length follows the stick-out: {dims:?}");
}

/// The check line says when the shank is in the clamping range, and names
/// the holder that refuses a shank outside it.
#[test]
fn a_shank_outside_the_clamping_range_is_refused_by_name() {
    let mut h = tall_window(TOOL);
    pick(&mut h, "BT30 ER20");
    assert!(
        shows(&h, &format!("{FITS} the shank Ø8 is in the clamping range 1 to 13 mm")),
        "an 8 mm shank fits ER20 collets: {:?}",
        painted(&h)
    );
    draft(&mut h).shank_d = 16.0;
    h.run_steps(2);
    assert!(shows(&h, &format!("{REFUSED} BT30 ER20:")), "the refusal names the holder");
    assert!(shows(&h, "does not fit"), "the refusal says why");
    assert!(!shows(&h, FITS), "no fit line beside the refusal");
}

/// The check line's text, in millimetres and in inches.
#[test]
fn the_clamp_check_states_the_range_or_the_refusal() {
    let Ok(e) = holder_library::find("BT30 ER20") else {
        panic!("BT30 ER20 is a library holder");
    };
    let ok = assembly_group::clamp_check("BT30 ER20", Ok(&e), 10.0, Unit::Millimeters);
    assert_eq!(ok, Ok(format!("{FITS} the shank Ø10 is in the clamping range 1 to 13 mm (ER20 collets).")));
    let Err(bad) = assembly_group::clamp_check("BT30 ER20", Ok(&e), 20.0, Unit::Millimeters) else {
        panic!("a 20 mm shank is outside ER20 collets");
    };
    assert!(bad.starts_with(&format!("{REFUSED} BT30 ER20: ")) && bad.contains("does not fit"), "{bad}");
    let Err(unknown) = assembly_group::clamp_check("BT30 Weldon", Err("no bore".into()), 7.0, Unit::Millimeters) else {
        panic!("an unresolved pick is refused");
    };
    assert_eq!(unknown, format!("{REFUSED} BT30 Weldon: no bore"));
    let inch = assembly_group::clamp_check("BT30 ER20", Ok(&e), 12.7, Unit::Inches);
    assert!(inch.as_deref().is_ok_and(|t| t.contains("Ø0.5 ") && t.contains(" in ")), "{inch:?}");
}

/// "Open holder" shows the picked holder in the Holders page; the tool's
/// draft (and its holder draft) stay, so the way back finds them.
#[test]
fn open_holder_shows_the_holder_view_and_keeps_the_draft() {
    let mut h = tall_window(TOOL);
    pick(&mut h, "BT30 ER20");
    holder(&mut h).projection = 61.0;
    h.run_steps(2);
    let link = format!("{OPEN_HOLDER} \"BT30 ER20\"");
    library_window(&h).get_by_label(&link).click();
    h.run_steps(3);
    let v = h.state().state.tool_library.as_ref().unwrap_or_else(|| panic!("the window stays open"));
    assert_eq!((v.page, v.holders.open.as_deref()), (LibraryPage::Holders, Some("BT30 ER20")), "the holder view opens");
    let kept = v.editor.as_ref().map(|e| (e.draft.name.clone(), e.holder.projection));
    assert_eq!(kept, Some((TOOL.to_owned(), 61.0)), "the tool's draft and its stick-out stay");
}

/// The side drawing is the default view. It is drawn to scale with the
/// spindle nose and the holder's name, its flute lines come from the
/// cached model, and a still frame builds no model.
#[test]
fn the_side_drawing_is_the_default_and_a_still_frame_builds_nothing() {
    let mut h = tall_window(TOOL);
    let face = h.state().state.tool_library.as_ref().and_then(|v| v.editor.as_ref()).map(|e| e.face.preview);
    assert_eq!(face, Some(PreviewMode::Side), "the side drawing is the default");
    pick(&mut h, "BT30 ER20");
    draft(&mut h).flutes = Some(4);
    draft(&mut h).helix_deg = "30".into();
    h.run_steps(2);
    let texts = painted(&h);
    for want in [format!("{} · mm", side_view::TO_SCALE), side_view::NOSE.to_owned(), "BT30 ER20".to_owned()] {
        assert!(texts.contains(&want), "the side drawing paints {want:?}: {texts:?}");
    }
    assert!(texts.iter().any(|t| t.starts_with("gauge length ")), "the gauge length is dimensioned: {texts:?}");
    let m = preview_model::last().unwrap_or_else(|| panic!("the preview built a model"));
    assert!(!m.flute_lines.is_empty(), "a 4-flute end mill has flute lines");
    let (builds, (drawn, _)) = (preview_model::builds(), side_view::drawn());
    h.run_steps(3);
    assert_eq!(preview_model::builds(), builds, "a still frame builds no model");
    assert!(side_view::drawn().0 > drawn, "the drawing is still painted");
}
