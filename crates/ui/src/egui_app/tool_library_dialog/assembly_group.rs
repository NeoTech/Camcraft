//! #3895 (U4 of #3663): the assembly group of the tool library window's
//! Tool tab - the holder pick (`tool_holder_picker`), the gauge length,
//! the projection (the stick-out), their read-only total, what Apply
//! holder would refuse (by name, before the click), a holder taper that
//! is not the machine's `tooling.spindle_mount`, and a projection that a
//! tool length change re-seeded (#808 item 2). The values are the
//! window's own holder draft (`ToolEditorView::holder`); only Apply
//! holder lands them, as the one undoable `SetToolBody::library`.
//! #3898 (U7): the component stack (`stack`) stands below the holder.
//! #4336 (U10): the group reads as the #3663 mock-up - the heading says
//! what it is for; Holder, Stick-out and the holder's gauge length are
//! typed; the gauge length is shown with its sum and an "Open holder" link
//! to the holder view; one check line says if the shank is in the holder's
//! clamping range, or names the holder that refuses it (`clamp_check`).
//! Provenance: our own group over `app::holder_library`; no HSMWorks
//! material involved.

use egui::{Color32, Ui};

use camcraft_app::holder_library::{self, Clamp, HolderEntry};
use camcraft_app::units::{self, Unit};

use crate::state::{State, is_spun_kind};

/// The group's heading, and its AccessKit name.
pub(crate) const HEADING: &str = "ASSEMBLY: HOW THIS TOOL SITS IN A HOLDER";
/// #4336: the read-only total, and the link to the holder view.
pub(crate) const GAUGE: &str = "Gauge length";
pub(crate) const OPEN_HOLDER: &str = "Open holder";
/// #4336: the first word of the clamping-range check line.
pub(crate) const FITS: &str = "Fits:";
pub(crate) const REFUSED: &str = "Refused by";
/// #3930: the folded body section's heading, and its AccessKit name.
pub(crate) const BODY: &str = "Body";

const WARN: Color32 = Color32::from_rgb(230, 170, 60);
const BAD: Color32 = Color32::from_rgb(230, 90, 80);

pub(super) fn draw(ui: &mut Ui, state: &mut State) {
    ui.add_space(8.0);
    ui.label(egui::RichText::new(HEADING).small().weak());
    ui.separator();
    if !state.tool_ui.exists {
        ui.label("Add the tool first, then choose its holder.");
        return;
    }
    let u = state.units();
    if is_spun_kind(state.tool_ui.tool_kind()) {
        super::super::tool_holder_picker::holder_fields(ui, state, u);
        notes(ui, state, u);
        super::super::tool_holder_picker::apply_holder_button(ui, state);
    } else {
        ui.label("A turning tool takes no library holder: import its holder in the Body section.");
    }
    // #3898: the tool's component stack (the document's shared parts).
    super::stack::draw(ui, state);
    body(ui, state, u);
}

/// #3930: the folded Body section - the stand-in holder cylinder, then
/// the body import, its placement and its pieces. Each landing is the
/// command it was in the tool shape window (`SetToolBody`, its piece
/// commands), or the window's Apply for the two stand-in numbers.
fn body(ui: &mut Ui, state: &mut State, u: Unit) {
    super::super::layout::group_open(ui, BODY, None, false, |ui| {
        super::super::tool_settings_grid::holder(ui, state);
        super::super::tool_body_page::body_section(ui, state, true, u);
    });
}

/// The total with its sum and the link, the clamping-range check, the
/// re-seed note, the mount warning and the refusal.
fn notes(ui: &mut Ui, state: &mut State, u: Unit) {
    let d = &state.tool_profile_ui;
    let Some(entry) = d.library.clone() else {
        return;
    };
    let (gauge, projection, reseeded) = (d.gauge, d.projection, d.reseeded);
    let l = |v: f64| format!("{} {}", short(v, u), u.suffix());
    super::super::layout::prop_row_hint(ui, GAUGE, "from the gauge line to the tool tip: the holder's gauge length plus the stick-out", |ui| {
        ui.label(l(gauge + projection))
    });
    let shank = state.tool_ui.shank_d;
    let resolved = holder_library::resolve(&entry, shank);
    let mut open = false;
    ui.horizontal_wrapped(|ui| {
        ui.weak(format!("= {} of the holder + {} stick-out.", l(gauge), l(projection)));
        if let Ok(e) = &resolved {
            open = ui.link(format!("{OPEN_HOLDER} \"{}\"", e.name())).on_hover_text("show this holder in the Holders page; the draft stays").clicked();
        }
    });
    let check = clamp_check(&entry, resolved.as_ref().map_err(ToString::to_string), shank, u);
    let inks = super::super::colorvision::text_inks(ui, &state.world);
    match &check {
        Ok(text) => ui.label(text),
        Err(text) => ui.colored_label(inks.error, text),
    };
    if reseeded {
        ui.colored_label(
            WARN,
            format!("The tool length changed: the stick-out follows the new tool ({}). Apply holder to store it.", l(projection)),
        );
    }
    let stored = state.editor_stored_tool();
    if let Some(taper) = resolved.as_ref().ok().and_then(|e| e.taper)
        && let Some(mount) = camcraft_app::machine::document_of(&state.world).1.and_then(|m| m.spindle_mount)
        && !mount.eq_ignore_ascii_case(taper.name)
    {
        ui.colored_label(WARN, format!("The machine's spindle takes {mount}: this holder has a {} taper.", taper.name));
    }
    // The check line names a shank refusal of the draft; this names the
    // rest of what Apply holder would refuse on the stored tool (another
    // shank, the lengths, the fit).
    let same_shank = stored.as_ref().is_some_and(|t| (t.shank_d - shank).abs() < 1e-9);
    // Both verdicts come from the cached preview model (built when its
    // inputs change). The refusal is the command's, on the stored tool;
    // the draft note judges the unapplied tool numbers only.
    let m = super::model::of(state);
    if let Some(err) = m.holder_err.clone().filter(|_| !(check.is_err() && same_shank)) {
        ui.colored_label(BAD, format!("Apply holder refuses this: {err}"));
    }
    if let Some(note) = m.draft_err.clone().filter(|_| check.is_ok()) {
        ui.colored_label(
            WARN,
            format!("Draft note: the unapplied tool numbers do not take this holder ({note}). Apply holder checks the stored tool."),
        );
    }
    if open && let Ok(e) = resolved {
        state.open_holder_in_library(Some(&e.name()));
    }
}

/// #4336: the clamping-range check line of the holder `pick` for a shank
/// of `shank_d` (`resolved`: the entry the pick resolves to, or why it
/// does not). `Ok` says that the shank is in range; `Err` names the holder
/// that refuses it, with the library's own reason.
pub(crate) fn clamp_check(pick: &str, resolved: Result<&HolderEntry, String>, shank_d: f64, u: Unit) -> Result<String, String> {
    let e = resolved.map_err(|err| format!("{REFUSED} {pick}: {err}"))?;
    if let Err(err) = e.fits_shank(shank_d) {
        return Err(format!("{REFUSED} {}: {err}", e.name()));
    }
    let s = short(shank_d, u);
    Ok(match e.clamp {
        Clamp::Er(er) => format!(
            "{FITS} the shank Ø{s} is in the clamping range {} to {} {} ({} collets).",
            short(er.min_bore, u),
            short(er.max_bore, u),
            u.suffix(),
            er.name
        ),
        Clamp::Weldon { bore, .. } | Clamp::Shrink { bore, .. } => {
            format!("{FITS} the shank Ø{s} takes the Ø{} {} bore of the holder.", short(bore, u), u.suffix())
        }
    })
}

/// A length in the document's unit without trailing zeros ("10", "12.5").
pub(crate) fn short(v: f64, u: Unit) -> String {
    let s = units::len(v, u, 3);
    if s.contains('.') { s.trim_end_matches('0').trim_end_matches('.').to_owned() } else { s }
}
