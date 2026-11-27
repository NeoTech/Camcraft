//! The axis chain as the file editor shows it (#2756): the rows, each
//! row's notes and the validator keys that show at a row. It is derived
//! once per edit of the draft and read back on every other frame (the
//! frame-cost rule, #2238): the draft, the file's one writer, drops it on
//! every write.
//!
//! Provenance: our own editor model, specced from issue #2756 and the
//! frame-cost rule in `crates/ui/AGENTS.md`.

use std::cell::Cell;
use std::sync::{Arc, OnceLock};

use serde_json::Value;

use super::chain::{self, AxisRow};

/// The axis chain's rows with what each row shows.
#[derive(Clone, Debug, PartialEq)]
pub struct ChainView {
    /// The rows in depth-first order, as `parse_mch` reads them.
    pub rows: Vec<AxisRow>,
    /// Each row's notes: a letter that is not the RS-274 one, an `offset`
    /// with no parent rotary. One list per row.
    pub notes: Vec<Vec<String>>,
    /// The validator keys that show at each row. One pair per row.
    pub keys: Vec<[String; 2]>,
}

impl ChainView {
    /// Derive the view from the file: one real build, counted.
    #[must_use]
    pub fn of(v: &Value) -> Self {
        BUILDS.with(|b| b.set(b.get() + 1));
        let rows = chain::rows(v);
        let notes = rows.iter().map(|r| r.letter_note().into_iter().chain(r.offset_note()).collect()).collect();
        let keys = rows.iter().map(|r| row_keys(&r.name)).collect();
        Self { rows, notes, keys }
    }

    /// Whether a refusal keyed `key` shows at an axis row rather than
    /// above the groups.
    #[must_use]
    pub fn at_a_row(&self, key: &str) -> bool {
        self.keys.iter().flatten().any(|k| k == key)
    }
}

/// The validator keys that belong on the row of axis `name`.
#[must_use]
pub fn row_keys(name: &str) -> [String; 2] {
    [name.to_owned(), format!("{name} axis")]
}

thread_local! {
    /// Real builds of [`ChainView::of`] on this thread.
    static BUILDS: Cell<u64> = const { Cell::new(0) };
}

/// How many real builds [`ChainView::of`] made on this thread: the
/// counter the frame-cost pins read (a still frame builds none).
#[must_use]
pub fn chain_view_builds() -> u64 {
    BUILDS.with(Cell::get)
}

/// The draft's memo of its view: built on the first read after a write,
/// emptied by every write.
#[derive(Clone, Debug, Default)]
pub(crate) struct Memo(OnceLock<Arc<ChainView>>);

impl Memo {
    /// The view of `v`, built only when the memo is empty.
    pub(crate) fn get(&self, v: &Value) -> Arc<ChainView> {
        Arc::clone(self.0.get_or_init(|| Arc::new(ChainView::of(v))))
    }

    /// Forget the view: the file is about to change.
    pub(crate) fn clear(&mut self) {
        self.0 = OnceLock::new();
    }
}

/// A memo is not part of what a draft is: two drafts with the same file
/// are equal whether or not either has built its view.
impl PartialEq for Memo {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::super::chain::{Place, Step};
    use super::super::draft::MachineDraft;
    use super::super::Kind;
    use super::*;

    fn builds_during(f: impl FnOnce()) -> u64 {
        let before = chain_view_builds();
        f();
        chain_view_builds() - before
    }

    #[test]
    fn a_read_with_no_write_between_builds_nothing() {
        let m = MachineDraft::blank(Kind::Mill);
        assert_eq!(builds_during(|| drop(m.chain_view())), 1, "the first read builds");
        assert_eq!(builds_during(|| (0..5).for_each(|_| drop(m.chain_view()))), 0, "later reads build nothing");
    }

    #[test]
    fn every_write_gives_the_view_a_fresh_build_gives() {
        let mut m = MachineDraft::blank(Kind::Mill);
        let fresh = |m: &MachineDraft| ChainView::of(m.value());
        type Write = Box<dyn Fn(&mut MachineDraft)>;
        let writes: [(&str, Write); 8] = [
            ("add a rotary", Box::new(|m| drop(m.add_rotary_axis(Place::OnTable, "B", [1.0, 0.0, 0.0])))),
            ("add another", Box::new(|m| drop(m.add_rotary_axis(Place::OnTable, "A", [1.0, 0.0, 0.0])))),
            ("move it up past B", Box::new(|m| {
                let at = m.chain_view().rows.iter().find(|r| r.name == "A").map(|r| r.pointer.clone()).unwrap_or_default();
                drop(m.move_axis(&at, Step::Up));
            })),
            ("turn it", Box::new(|m| {
                let at = m.chain_view().rows.iter().find(|r| r.rotary).map(|r| r.pointer.clone()).unwrap_or_default();
                drop(m.set_rotary_axis(&at, [0.0, 1.0, 0.0]));
            })),
            ("reverse X", Box::new(|m| {
                let at = m.chain_view().rows[0].pointer.clone();
                drop(m.set_axis_reversed(&at, true));
            })),
            ("add a linear", Box::new(|m| drop(m.add_linear_axis(Place::OnTable, "U")))),
            ("remove it", Box::new(|m| {
                let at = m.chain_view().rows.iter().find(|r| r.name == "U").map(|r| r.pointer.clone()).unwrap_or_default();
                drop(m.remove_axis(&at));
            })),
            ("write the JSON by hand", Box::new(|m| {
                let at = m.chain_view().rows.iter().find(|r| r.rotary).map(|r| r.pointer.clone()).unwrap_or_default();
                if let Some(p) = m.value_mut().pointer_mut(&at) {
                    p["offset"] = serde_json::json!([0.0, 0.0, 10.0]);
                }
            })),
        ];
        for (what, write) in writes {
            let before = m.chain_view();
            write(&mut m);
            let after = m.chain_view();
            assert_eq!(*after, fresh(&m), "{what}: the memo is the fresh build");
            assert_ne!(*after, *before, "{what}: the write changed what the rows show");
        }
        assert!(m.chain_view().notes.iter().flatten().any(|n| n.contains("states offset")), "the hand-written offset shows its note");
        // A write elsewhere in the file drops the view too: the draft does
        // not judge which keys the rows read.
        assert_eq!(builds_during(|| {
            drop(m.set("/general/model", super::super::fields::FieldKind::Text, "Mill 2"));
            drop(m.chain_view());
        }), 1);
    }
}
