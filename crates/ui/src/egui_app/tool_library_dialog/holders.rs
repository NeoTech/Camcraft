//! #3897 (U6 of #3663): the Holders page of the tool library window - the
//! built-in holder library (`holder_library::library`) grouped by spindle
//! taper, the taper of the mounted machine first and marked. A row gives
//! the name, the taper, the clamp, the clamping range, the gauge length,
//! the largest diameter and the number of document tools that use the
//! holder (`app::tool_body::holder_users`, keyed on the entry name). A
//! click on a name opens the read-only holder view (`holder_view`).
//! Built-in holders are read-only. #3898 (U7): the user holders of the
//! document (its component table) head the list; a click opens one in
//! the component editor, and a built-in holder's view copies it as a
//! user holder.
//! #4338 (U12): one header row - the search (taper, clamp, bore or collet
//! size and name, each term must match; "N of M holders"; Esc clears it),
//! New holder, Import and Export. New holder makes a user holder of the
//! machine's taper and opens it in the holder form (`holder_form`).
//! Frame cost: the library rows and their search words are built once for
//! each thread; the search result is kept for its input
//! (`state::holder_hits`); a frame reads the document's holder references
//! and formats the rows.
//! Provenance: our own list over our own holder library; no HSMWorks
//! material involved.

use std::cell::OnceCell;
use std::collections::HashMap;
use std::rc::Rc;

use egui::{Key, RichText, Ui, WidgetInfo, WidgetType};

use camcraft_app::component::ComponentKind;
use camcraft_app::component_library::BUILTIN_PREFIX;
use camcraft_app::connection_code::ConnectionCode;
use camcraft_app::holder_library::{self, Clamp, ER_SIZES, HolderEntry};
use camcraft_app::tool_body;
use camcraft_app::units::{self, Unit};

use crate::state::{ComponentAction, ComponentGroup, ComponentRow, ErrorScope, HolderHits, HolderItem, State, holder_hits};

/// The label of the row that shows every taper.
pub(crate) const ALL: &str = "All holders";
/// The name of the group of the straight-shank chucks (no taper).
pub(crate) const STRAIGHT: &str = "Straight shank";
/// The note that says what the page cannot do yet.
pub(crate) const READ_ONLY: &str =
    "Built-in holders are read-only. Open one and copy it as a user holder to edit it.";
/// #3898: the heading of the document's own holders.
pub(crate) const USER_HOLDERS: &str = "USER HOLDERS";
/// #3898: the button on a built-in holder that makes a user holder of it.
pub(crate) const COPY: &str = "Copy as user holder";
/// #4338: the search box (its AccessKit name) and its hint.
pub(crate) const SEARCH: &str = "Holder search";
const SEARCH_HINT: &str = "Search taper, clamp, size or name";
/// #4338: the header row's buttons.
pub(crate) const NEW: &str = "New holder…";
pub(crate) const IMPORT: &str = "Import…";
pub(crate) const EXPORT: &str = "Export…";
/// Why Import and Export are not offered yet.
pub(crate) const NO_FILES: &str = "holder files follow in #4389";
/// The column heads.
pub(crate) const HEADS: [&str; 7] = ["Name", "Taper", "Clamp", "Clamps Ø", "Gauge length", "Largest Ø", "Used by"];
const TAPER_W: f32 = 170.0;

/// One library holder as the list shows it.
#[derive(Clone, Debug)]
pub(crate) struct Row {
    pub entry: HolderEntry,
    pub name: String,
    /// The taper's name, or "" for a straight-shank chuck.
    pub taper: &'static str,
}

thread_local! {
    static ROWS: OnceCell<Rc<Vec<Row>>> = const { OnceCell::new() };
    static ITEMS: OnceCell<Rc<Vec<HolderItem>>> = const { OnceCell::new() };
}

/// The library rows' search words, in the rows' order (built once).
pub(crate) fn items() -> Rc<Vec<HolderItem>> {
    ITEMS.with(|i| {
        Rc::clone(i.get_or_init(|| {
            Rc::new(
                rows()
                    .iter()
                    .map(|r| {
                        let (from, to) = super::holder_view::clamps(&r.entry);
                        let words = format!("{} {} {}", r.name, group_name(r.taper), clamp_text(&r.entry)).to_lowercase();
                        HolderItem { words, from, to, taper: r.taper.to_owned() }
                    })
                    .collect(),
            )
        }))
    })
}

/// A document holder or adapter as the search reads it: its name, its
/// sides and the shank range its workpiece side takes.
fn user_item((r, _, _): &ComponentRow) -> Option<HolderItem> {
    if !matches!(crate::state::group_of(&r.component), ComponentGroup::Holders | ComponentGroup::Adapters) {
        return None;
    }
    let (machine, tool) = super::components::sides(&r.component);
    let (from, to) = match r.component.kind() {
        ComponentKind::AdaptiveItem { csw: ConnectionCode::Er { size }, .. } => {
            let name = format!("ER{size}");
            ER_SIZES.iter().find(|e| e.name == name).map_or((-1.0, -1.0), |e| (e.min_bore, e.max_bore))
        }
        ComponentKind::AdaptiveItem { csw: ConnectionCode::Weldon { d } | ConnectionCode::Cylindrical { d }, .. } => (*d, *d),
        _ => (-1.0, -1.0),
    };
    let words = format!("{} {machine} {tool}", r.component.name()).to_lowercase();
    Some(HolderItem { words, from, to, taper: machine })
}

/// The library's rows, in the library's order (built once).
pub(crate) fn rows() -> Rc<Vec<Row>> {
    ROWS.with(|r| {
        Rc::clone(r.get_or_init(|| {
            Rc::new(
                holder_library::library()
                    .into_iter()
                    .map(|e| Row { name: e.name(), taper: e.taper.map_or("", |t| t.name), entry: e })
                    .collect(),
            )
        }))
    })
}

/// The group name of a taper ("" is the straight-shank group).
pub(crate) fn group_name(taper: &str) -> &str {
    if taper.is_empty() { STRAIGHT } else { taper }
}

/// The taper the mounted machine's spindle takes, if the machine states one.
pub(crate) fn machine_taper(state: &State) -> Option<String> {
    camcraft_app::machine::document_of(&state.world).1.and_then(|m| m.spindle_mount)
}

/// The tapers in the list's order: the machine's first, then the library's.
pub(crate) fn tapers(rows: &[Row], machine: Option<&str>) -> Vec<&'static str> {
    let mut out: Vec<&'static str> = Vec::new();
    for r in rows {
        if !out.contains(&r.taper) {
            out.push(r.taper);
        }
    }
    if let Some(m) = machine
        && let Some(i) = out.iter().position(|t| t.eq_ignore_ascii_case(m))
    {
        let t = out.remove(i);
        out.insert(0, t);
    }
    out
}

/// The document tools that use each holder, by `tool_body::holder_key`.
pub(crate) fn users(state: &State) -> HashMap<String, Vec<String>> {
    let mut out: HashMap<String, Vec<String>> = HashMap::new();
    for (tool, r) in tool_body::holder_users(&state.world) {
        out.entry(tool_body::holder_key(&r.entry)).or_default().push(tool);
    }
    out
}

/// The clamp as words: "ER32 collet", "Weldon side lock", "shrink fit".
pub(crate) fn clamp_text(e: &HolderEntry) -> String {
    match e.clamp {
        Clamp::Er(er) => format!("{} collet", er.name),
        Clamp::Weldon { .. } => "Weldon side lock".to_owned(),
        Clamp::Shrink { .. } => "shrink fit".to_owned(),
    }
}

/// The shank diameters the holder takes.
pub(crate) fn range_text(e: &HolderEntry, u: Unit) -> String {
    match e.clamp {
        Clamp::Er(er) => format!("{} – {}", units::len(er.min_bore, u, 2), units::len(er.max_bore, u, 2)),
        Clamp::Weldon { bore, .. } | Clamp::Shrink { bore, .. } => units::len(bore, u, 3),
    }
}

/// The largest diameter of the holder: the flange's or the nose's.
pub(crate) fn largest_d(e: &HolderEntry) -> f64 {
    super::holder_drawing::largest_d(e)
}

/// The built-in holder New holder copies: the machine taper's ER32 chuck
/// (or its first holder), else BT40 ER32.
pub(crate) fn new_holder_id(rows: &[Row], machine: Option<&str>) -> String {
    let of = |t: &str| {
        let mine: Vec<&Row> = rows.iter().filter(|r| r.taper.eq_ignore_ascii_case(t)).collect();
        mine.iter().find(|r| matches!(r.entry.clamp, Clamp::Er(er) if er.name == "ER32")).or(mine.first()).map(|r| r.name.clone())
    };
    let name = machine.and_then(of).unwrap_or_else(|| "BT40 ER32".to_owned());
    format!("{BUILTIN_PREFIX}holder:{name}")
}

/// "N of M holders".
pub(crate) fn count_text(hits: &HolderHits) -> String {
    format!("{} of {} holders", hits.shown(), hits.total())
}

pub(super) fn draw(ui: &mut Ui, state: &mut State) {
    let rows = rows();
    let machine = machine_taper(state);
    let users = users(state);
    let Some(view) = state.tool_library.as_ref().map(|v| v.holders.clone()) else {
        return;
    };
    let derived = state.component_derived(None);
    let hits = holder_hits(&view.filter, view.taper.as_deref(), &items(), &derived.rows, user_item);
    let mut pick_taper = None;
    let mut open = None;
    let mut edit = None;
    let mut new = false;
    let mut filter = view.filter.clone();
    ui.horizontal_top(|ui| {
        ui.vertical(|ui| {
            ui.set_width(TAPER_W);
            pick_taper = taper_column(ui, &rows, machine.as_deref(), view.taper.as_deref());
        });
        ui.separator();
        ui.vertical(|ui| {
            new = header_row(ui, &mut filter, &hits);
            if let Some(msg) = state.error_for(ErrorScope::Tool).map(str::to_owned) {
                super::super::error_text::error_text(ui, super::super::colorvision::text_inks(ui, &state.world).error, &msg);
            }
            edit = user_holders(ui, &derived.rows, &hits);
            egui::ScrollArea::both().id_salt("tool_library_holders").auto_shrink([false, false]).show(ui, |ui| {
                open = table(ui, &rows, &users, machine.as_deref(), &hits, state.units());
            });
        });
    });
    if let Some(v) = state.tool_library.as_mut() {
        v.holders.filter = filter;
        if let Some(t) = pick_taper {
            v.holders.taper = t;
        }
    }
    if let Some(name) = open {
        state.open_holder_in_library(Some(&name));
    }
    if let Some(id) = edit {
        super::component_edit::open(state, &id);
    }
    if new {
        state.queue_component(ComponentAction::CopyAsUser { id: new_holder_id(&rows, machine.as_deref()) });
    }
}

/// #4338: the search box with its count, then New holder, Import and
/// Export. Esc in the box clears it. Gives whether New holder was clicked.
fn header_row(ui: &mut Ui, filter: &mut String, hits: &HolderHits) -> bool {
    let mut new = false;
    ui.horizontal(|ui| {
        let w = (ui.available_width() - 420.0).max(140.0);
        let r = ui.add(egui::TextEdit::singleline(filter).hint_text(SEARCH_HINT).desired_width(w));
        r.widget_info(|| WidgetInfo::labeled(WidgetType::TextEdit, true, SEARCH));
        let r = r.on_hover_text("each word must match: the taper, the clamp, a bore or collet size, the name (\"BT40 ER32\", \"shrink 12\"); Esc clears");
        if (r.has_focus() || r.lost_focus()) && ui.input(|i| i.key_pressed(Key::Escape)) {
            filter.clear();
        }
        ui.weak(count_text(hits));
        new = ui.button(NEW).on_hover_text("a user holder of the machine's taper, in this document, that you can edit (one undo step)").clicked();
        let _ = ui.add_enabled(false, egui::Button::new(IMPORT)).on_disabled_hover_text(NO_FILES);
        let _ = ui.add_enabled(false, egui::Button::new(EXPORT)).on_disabled_hover_text(NO_FILES);
    });
    new
}

/// #3898: the document's holders and adapters (its component table), each
/// with its library status and the tools whose stack uses it. A click on
/// a name opens it in the component editor. Gives the id to edit.
fn user_holders(ui: &mut Ui, rows: &[ComponentRow], hits: &HolderHits) -> Option<String> {
    // The document holds no holder: the list shows the library alone.
    if hits.user_total == 0 {
        return None;
    }
    ui.label(RichText::new(USER_HOLDERS).small().weak());
    let mut edit = None;
    egui::Grid::new("tool_library_user_holders").striped(true).show(ui, |ui| {
        for h in ["Name", "Machine side", "Tool side", "Library", "Used by"] {
            ui.label(RichText::new(h).strong());
        }
        ui.end_row();
        for (r, status, users) in hits.user.iter().filter_map(|n| rows.get(*n)) {
            if ui.selectable_label(false, r.component.name()).on_hover_text("edit this holder").clicked() {
                edit = Some(r.component.id().to_owned());
            }
            let (machine, tool) = super::components::sides(&r.component);
            ui.label(machine);
            ui.label(tool);
            ui.label(super::components::status_text(status));
            ui.label(super::holder_view::count(users.len()));
            ui.end_row();
        }
    });
    ui.add_space(6.0);
    edit
}

/// "All holders", then one row for each taper with its count, then the
/// machine's taper. Gives the new taper pick when the user clicks a row.
fn taper_column(ui: &mut Ui, rows: &[Row], machine: Option<&str>, picked: Option<&str>) -> Option<Option<String>> {
    let mut out = None;
    ui.label(RichText::new("SPINDLE TAPER").small().weak());
    let item = |ui: &mut Ui, sel: bool, text: &str, n: usize| {
        ui.horizontal(|ui| {
            let c = ui.selectable_label(sel, text).clicked();
            ui.weak(n.to_string());
            c
        })
        .inner
    };
    if item(ui, picked.is_none(), ALL, rows.len()) {
        out = Some(None);
    }
    for t in tapers(rows, machine) {
        let n = rows.iter().filter(|r| r.taper == t).count();
        let mark = machine.is_some_and(|m| m.eq_ignore_ascii_case(t));
        let text = if mark { format!("{} (machine)", group_name(t)) } else { group_name(t).to_owned() };
        if item(ui, picked == Some(t), &text, n) {
            out = Some(Some(t.to_owned()));
        }
    }
    ui.add_space(8.0);
    ui.label(RichText::new("MACHINE").small().weak());
    ui.label(match machine {
        Some(m) => format!("The mounted machine takes {m}. Its holders come first."),
        None => "The mounted machine states no spindle taper.".to_owned(),
    });
    out
}

/// The holder rows the search shows, grouped by taper; gives the name the
/// user opened.
fn table(ui: &mut Ui, rows: &[Row], users: &HashMap<String, Vec<String>>, machine: Option<&str>, hits: &HolderHits, u: Unit) -> Option<String> {
    let mut open = None;
    egui::Grid::new("tool_library_holder_rows").striped(true).show(ui, |ui| {
        for h in HEADS {
            ui.label(RichText::new(h).strong());
        }
        ui.end_row();
        for t in tapers(rows, machine) {
            let shown: Vec<&Row> = hits.library.iter().filter_map(|n| rows.get(*n)).filter(|r| r.taper == t).collect();
            if shown.is_empty() {
                continue;
            }
            let mark = if machine.is_some_and(|m| m.eq_ignore_ascii_case(t)) { "  (the machine's taper)" } else { "" };
            ui.label(RichText::new(format!("{}{mark}", group_name(t))).small().weak());
            ui.end_row();
            for r in shown {
                if ui.selectable_label(false, &r.name).on_hover_text("open this holder").clicked() {
                    open = Some(r.name.clone());
                }
                ui.label(group_name(r.taper));
                ui.label(clamp_text(&r.entry));
                ui.label(range_text(&r.entry, u));
                ui.label(units::len(r.entry.gauge_len, u, 2));
                ui.label(units::len(largest_d(&r.entry), u, 2));
                let n = users.get(&tool_body::holder_key(&r.name)).map_or(0, Vec::len);
                ui.label(super::holder_view::count(n));
                ui.end_row();
            }
        }
    });
    open
}
