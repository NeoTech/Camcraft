//! The axis chain of a machine file being edited (#2756): add, move or
//! remove a linear or rotary part in `kinematics.default.parts`, reverse
//! a linear (#373) and turn a rotary about a new axis.
//!
//! Provenance: our own editor model, specced from issue #2756 and the
//! `.mch` grammar in `assets/machines/AGENTS.md` (axis parts nest; a
//! rotary's `parent` and `chain_order` follow the depth-first nesting,
//! #1962; a rotary with no head below it turns the work, #804). The
//! rotary letters follow RS-274 (A, B, C about X, Y, Z).
//!
//! Every edit works on the file's JSON in place, so a key the editor
//! does not know is kept. A refused edit changes nothing.

use serde_json::{json, Value};

use super::fields;
use super::validate::axis_parts;

const ROOT: &str = "/kinematics/default/parts";
const CONTROLLER: &str = "/controller/synced_configuration/parts";
const LINEAR_NAMES: [&str; 6] = ["X", "Y", "Z", "U", "V", "W"];
const ROTARY_NAMES: [&str; 3] = ["A", "B", "C"];

/// Where a new axis part goes in the tree.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Place {
    /// Beside the axes on the machine's table part, before its head:
    /// a rotary here turns the work (a 4th axis bolted to the bed).
    OnTable,
    /// Between the head and what carried it: the head now rides on
    /// the new part, so a rotary here tilts the tool.
    CarriesHead,
}

/// Add a linear axis part named `name` (X, Y, Z, U, V or W). Returns
/// the new part's JSON pointer.
pub fn add_linear(v: &mut Value, place: Place, name: &str) -> Result<String, String> {
    if !LINEAR_NAMES.contains(&name) {
        return Err(format!("axis {name}: a linear axis is named X, Y, Z, U, V or W"));
    }
    let part = json!({ "id": free_id(v, name), "name": name, "type": "linear" });
    insert(v, place, name, part)
}

/// Add a rotary axis part named `name` (A, B or C) that turns about
/// `axis`. Returns the new part's JSON pointer.
pub fn add_rotary(v: &mut Value, place: Place, name: &str, axis: [f64; 3]) -> Result<String, String> {
    if !ROTARY_NAMES.contains(&name) {
        return Err(format!("axis {name}: a rotary is named A, B or C"));
    }
    check_direction(name, axis)?;
    let part = json!({ "id": free_id(v, name), "name": name, "type": "rotary", "axis": axis });
    insert(v, place, name, part)
}

/// Remove the axis part at `pointer`. The parts it carried take its
/// place in its list, in their order, so removing Z of a Z-Y-X stack
/// keeps Y and X, and removing a rotary that carries the head puts the
/// head back. The axis's controller row goes with it.
pub fn remove(v: &mut Value, pointer: &str) -> Result<(), String> {
    let not_one = || format!("{pointer}: not an axis part");
    let (list, index) = pointer.rsplit_once('/').ok_or_else(not_one)?;
    let index: usize = index.parse().map_err(|_| not_one())?;
    let in_tree = list == ROOT || (list.starts_with(&format!("{ROOT}/")) && list.ends_with("/parts"));
    if !in_tree {
        return Err(not_one());
    }
    let name = match v.pointer(pointer) {
        Some(p) if is_axis(p) => p.get("name").and_then(Value::as_str).map(str::to_owned),
        _ => return Err(not_one()),
    };
    let Some(Value::Array(a)) = v.pointer_mut(list) else { return Err(not_one()) };
    let mut gone = a.remove(index);
    let carried = match gone.get_mut("parts").map(Value::take) {
        Some(Value::Array(c)) => c,
        _ => Vec::new(),
    };
    a.splice(index..index, carried);
    if let Some(name) = name
        && let Some(Value::Object(ctrl)) = v.pointer_mut(CONTROLLER)
    {
        ctrl.remove(&name);
    }
    Ok(())
}

/// One step along the chain's rows (depth-first order).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    /// Toward the first row (nearer the root of the tree).
    Up,
    /// Toward the last row.
    Down,
}

/// Move the axis part at `pointer` one row along the chain: it trades
/// places with its neighbour row. The tree keeps its shape - which
/// slot carries the head, which sit on the table - and each axis keeps
/// all its own keys (`parts` stays with the slot). So a rotary that
/// moves up gets a new `parent` and `chain_order` (#1962). The neighbour
/// must be of the same kind, rotary or linear, and move the same thing,
/// the work or the tool. Returns where the moved
/// axis is now.
pub fn move_axis(v: &mut Value, pointer: &str, step: Step) -> Result<String, String> {
    let rows = axis_pointers(v);
    let i = rows.iter().position(|p| p == pointer).ok_or_else(|| format!("{pointer}: not an axis part"))?;
    let name = v.pointer(pointer).and_then(|p| p.get("name")).and_then(Value::as_str).unwrap_or(pointer);
    let j = match step {
        Step::Up => i.checked_sub(1).ok_or_else(|| format!("axis {name}: already first in the chain"))?,
        Step::Down => Some(i + 1).filter(|j| *j < rows.len()).ok_or_else(|| format!("axis {name}: already last in the chain"))?,
    };
    // A rotary in a linear's slot describes another machine (an A riding
    // on Y); a move only reorders axes of one kind (#2756, Q2).
    let kind = |at: &str| v.pointer(at).and_then(|p| p.get("type")).and_then(Value::as_str);
    if kind(&rows[i]) != kind(&rows[j]) {
        let what = if kind(&rows[i]) == Some("rotary") { "rotary" } else { "linear" };
        return Err(format!(
            "axis {name}: a {what} trades places only with a {what}; to put it elsewhere, remove it and add it again"
        ));
    }
    // The slot that carries the head keeps it, so a trade across it would
    // turn a table rotary into one that tilts the tool.
    let tool = |at: &str| first_of_type(v, &format!("{at}/parts"), "head").is_some();
    if tool(&rows[i]) != tool(&rows[j]) {
        let what = if tool(&rows[i]) { "tool" } else { "work" };
        return Err(format!(
            "axis {name}: an axis that moves the {what} trades places only with one that moves the {what}; to put it elsewhere, remove it and add it again"
        ));
    }
    // Depth-first, the earlier row is an ancestor or in an earlier
    // branch: write it first, so the later write lands in the new tree.
    let (first, second) = (&rows[i.min(j)], &rows[i.max(j)]);
    let (a, b) = match (v.pointer(first), v.pointer(second)) {
        (Some(a), Some(b)) => (a.clone(), b.clone()),
        _ => return Err(format!("{pointer}: not an axis part")),
    };
    let (Some(new_first), Some(new_second)) = (with_parts_of(&b, &a), with_parts_of(&a, &b)) else {
        return Err(format!("{pointer}: not an axis part"));
    };
    for (at, x) in [(first, new_first), (second, new_second)] {
        *v.pointer_mut(at).ok_or_else(|| format!("{at}: not an axis part"))? = x;
    }
    Ok(rows[j].clone())
}

/// `axis` with the `parts` of `slot` in place of its own.
fn with_parts_of(axis: &Value, slot: &Value) -> Option<Value> {
    let mut out = axis.as_object()?.clone();
    out.remove("parts");
    if let Some(parts) = slot.get("parts") {
        out.insert("parts".to_owned(), parts.clone());
    }
    Some(Value::Object(out))
}

/// One row of the editor's axis chain.
#[derive(Clone, Debug, PartialEq)]
pub struct AxisRow {
    /// Where the part lives: a JSON pointer into the file.
    pub pointer: String,
    /// The part's letter (its `name`, else its `id`).
    pub name: String,
    pub rotary: bool,
    /// A head is below the part: it moves the tool, not the work.
    pub carries_head: bool,
    /// A linear whose `+` moves the other way: `"direction": -1` (#373).
    pub reversed: bool,
    /// A rotary's `axis` as three finite numbers; `None` on a linear or
    /// when the file states no readable axis.
    pub axis: Option<[f64; 3]>,
    /// The rotary that carries this one (its nearest rotary above in
    /// the tree, #1962), by letter; `None` on a linear or a first rotary.
    pub parent: Option<String>,
    /// The part states `offset` (#1962).
    pub states_offset: bool,
}

impl AxisRow {
    /// The note a rotary row shows when its letter is not the one RS-274
    /// gives its axis (an `A` about Y). `None` when they agree, when the
    /// axis is tilted (no letter fits), or on a linear.
    #[must_use]
    pub fn letter_note(&self) -> Option<String> {
        let axis = self.axis?;
        let want = suggested_letter(axis)?;
        if self.name.eq_ignore_ascii_case(want) {
            return None;
        }
        let about = ["X", "Y", "Z"][ROTARY_NAMES.iter().position(|n| *n == want)?];
        Some(format!("{} turns about {about}: by RS-274 an axis about {about} is {want}", self.name))
    }

    /// The note a rotary row shows when it states `offset` but no rotary
    /// carries it: `offset` is measured from a parent rotary (#1962), so
    /// it has no reference (#2756, Q1). The validator does not refuse it.
    #[must_use]
    pub fn offset_note(&self) -> Option<String> {
        (self.rotary && self.states_offset && self.parent.is_none()).then(|| {
            format!("{} states offset, but no rotary carries it: offset is measured from a parent rotary", self.name)
        })
    }
}

/// The letter RS-274 gives a rotary about `axis`: about X is `A`, about
/// Y is `B`, about Z is `C`, either sign. `None` for a tilted axis (more
/// than one component) or a zero one.
#[must_use]
pub fn suggested_letter(axis: [f64; 3]) -> Option<&'static str> {
    let mut on = axis.iter().enumerate().filter(|(_, x)| **x != 0.0);
    match (on.next(), on.next()) {
        (Some((i, x)), None) if x.is_finite() => ROTARY_NAMES.get(i).copied(),
        _ => None,
    }
}

/// Reverse the linear axis at `pointer` (`"direction": -1`, #373), or
/// set it back to standard by removing the key: unstated is standard.
pub fn set_reversed(v: &mut Value, pointer: &str, reversed: bool) -> Result<(), String> {
    let (name, rotary) = axis_at(v, pointer)?;
    if rotary {
        return Err(format!("axis {name}: a rotary is not reversed; turn its axis instead"));
    }
    let Some(Value::Object(o)) = v.pointer_mut(pointer) else { return Err(format!("{pointer}: not an axis part")) };
    if reversed {
        o.insert("direction".to_owned(), json!(-1));
    } else {
        o.remove("direction");
    }
    Ok(())
}

/// Turn the rotary at `pointer` about `axis`. Refused by name on a
/// linear and on a zero or non-finite direction; its other keys stay.
pub fn set_axis(v: &mut Value, pointer: &str, axis: [f64; 3]) -> Result<(), String> {
    let (name, rotary) = axis_at(v, pointer)?;
    if !rotary {
        return Err(format!("axis {name}: a linear axis has no rotary axis"));
    }
    check_direction(&name, axis)?;
    put(v, pointer, "axis", json!(axis))
}

/// The letter and kind of the axis part at `pointer`.
fn axis_at(v: &Value, pointer: &str) -> Result<(String, bool), String> {
    let p = v.pointer(pointer).filter(|p| is_axis(p)).ok_or_else(|| format!("{pointer}: not an axis part"))?;
    let name = p.get("name").and_then(Value::as_str).or_else(|| p.get("id").and_then(Value::as_str)).unwrap_or(pointer);
    Ok((name.to_owned(), p.get("type").and_then(Value::as_str) == Some("rotary")))
}

fn check_direction(name: &str, axis: [f64; 3]) -> Result<(), String> {
    if !axis.iter().all(|x| x.is_finite()) {
        return Err(format!("axis {name}: the axis direction must be three finite numbers"));
    }
    if axis.iter().all(|x| *x == 0.0) {
        return Err(format!("axis {name}: the axis direction is zero"));
    }
    Ok(())
}

/// The axis parts in depth-first order, as `parse_mch` reads them.
#[must_use]
pub fn rows(v: &Value) -> Vec<AxisRow> {
    let mut rows: Vec<AxisRow> = axis_pointers(v)
        .into_iter()
        .filter_map(|pointer| {
            let p = v.pointer(&pointer)?;
            let name = p.get("name").and_then(Value::as_str).or_else(|| p.get("id").and_then(Value::as_str))?.to_owned();
            let rotary = p.get("type").and_then(Value::as_str) == Some("rotary");
            let carries_head = first_of_type(v, &format!("{pointer}/parts"), "head").is_some();
            let reversed = !rotary && p.get("direction").and_then(Value::as_f64) == Some(-1.0);
            let axis = p
                .get("axis")
                .and_then(Value::as_array)
                .filter(|a| rotary && a.len() == 3)
                .and_then(|a| a.iter().map(Value::as_f64).collect::<Option<Vec<f64>>>())
                .and_then(|d| <[f64; 3]>::try_from(d).ok())
                .filter(|d| d.iter().all(|x| x.is_finite()));
            let states_offset = p.get("offset").is_some_and(|o| !o.is_null());
            Some(AxisRow { pointer, name, rotary, carries_head, reversed, axis, parent: None, states_offset })
        })
        .collect();
    // Depth-first, a rotary's parent is the last rotary row before it
    // whose part holds it.
    for i in 0..rows.len() {
        if !rows[i].rotary {
            continue;
        }
        let inside = format!("{}/", rows[i].pointer);
        rows[i].parent = rows[..i]
            .iter()
            .rev()
            .find(|p| p.rotary && inside.starts_with(&format!("{}/", p.pointer)))
            .map(|p| p.name.clone());
    }
    rows
}

/// Every axis part's pointer, depth-first: the editor's rows.
fn axis_pointers(v: &Value) -> Vec<String> {
    fn walk(v: &Value, at: &str, out: &mut Vec<String>) {
        let Some(arr) = v.pointer(at).and_then(Value::as_array) else { return };
        for (i, p) in arr.iter().enumerate() {
            let here = format!("{at}/{i}");
            if is_axis(p) {
                out.push(here.clone());
            }
            walk(v, &format!("{here}/parts"), out);
        }
    }
    let mut out = Vec::new();
    walk(v, ROOT, &mut out);
    out
}

fn is_axis(p: &Value) -> bool {
    matches!(p.get("type").and_then(Value::as_str), Some("linear" | "rotary"))
}

fn insert(v: &mut Value, place: Place, name: &str, mut part: Value) -> Result<String, String> {
    if axis_parts(v).iter().any(|(n, _, _)| n.eq_ignore_ascii_case(name)) {
        return Err(format!("axis {name}: the machine already has an axis {name}"));
    }
    match place {
        Place::OnTable => {
            let table = first_of_type(v, ROOT, "table").ok_or("this machine has no table part to carry an axis")?;
            let list = format!("{table}/parts");
            if !v.pointer(&list).is_some_and(Value::is_array) {
                put(v, &table, "parts", json!([]))?;
            }
            let Some(Value::Array(a)) = v.pointer_mut(&list) else { return Err(format!("{list}: not a part list")) };
            let at = a
                .iter()
                .position(|p| p.get("type").and_then(Value::as_str) == Some("head"))
                .unwrap_or(a.len());
            a.insert(at, part);
            Ok(format!("{list}/{at}"))
        }
        Place::CarriesHead => {
            let head = fields::heads(v).into_iter().next().ok_or("this machine has no head for an axis to carry")?;
            let slot = v.pointer_mut(&head).ok_or_else(|| format!("{head}: no head part"))?;
            let head_part = slot.take();
            part["parts"] = json!([head_part]);
            *slot = part;
            Ok(head)
        }
    }
}

/// The pointer of the first part of type `ty` in the list at `at`,
/// depth-first.
fn first_of_type(v: &Value, at: &str, ty: &str) -> Option<String> {
    let arr = v.pointer(at)?.as_array()?;
    arr.iter().enumerate().find_map(|(i, p)| {
        let here = format!("{at}/{i}");
        if p.get("type").and_then(Value::as_str) == Some(ty) {
            Some(here)
        } else {
            first_of_type(v, &format!("{here}/parts"), ty)
        }
    })
}

fn put(v: &mut Value, at: &str, key: &str, x: Value) -> Result<(), String> {
    match v.pointer_mut(at) {
        Some(Value::Object(o)) => {
            o.insert(key.to_owned(), x);
            Ok(())
        }
        _ => Err(format!("{at}: not a part")),
    }
}

/// A part `id` no other part in the tree uses: the letter in lower
/// case, then with a number after it.
fn free_id(v: &Value, name: &str) -> String {
    let mut ids = Vec::new();
    collect_ids(v.pointer(ROOT), &mut ids);
    let base = name.to_ascii_lowercase();
    std::iter::once(base.clone())
        .chain((2u32..).map(|n| format!("{base}{n}")))
        .find(|id| !ids.contains(id))
        .unwrap_or(base)
}

fn collect_ids(parts: Option<&Value>, out: &mut Vec<String>) {
    let Some(arr) = parts.and_then(Value::as_array) else { return };
    for p in arr {
        if let Some(id) = p.get("id").and_then(Value::as_str) {
            out.push(id.to_owned());
        }
        collect_ids(p.get("parts"), out);
    }
}
