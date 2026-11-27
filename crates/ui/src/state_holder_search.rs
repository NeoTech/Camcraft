//! #4338 (U12 of #3663): the search of the Holders page. The text splits
//! into terms; a holder is shown when each term is in its words (name,
//! taper, clamp, sizes) or, for a number, when the holder clamps a shank
//! of that size - "BT40 ER32", "shrink 12". The result is kept for its
//! input (the text, the taper pick, the library rows and the document's
//! component rows, each an `Rc` that a change replaces): a still frame
//! searches nothing (`holder_searches`).
//! Provenance: our own search over our own holder rows; no HSMWorks
//! material involved.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use super::ComponentRow;

/// One searchable holder: its words (lower case), the shank range it
/// clamps (mm) and its taper's name ("" for none).
#[derive(Clone, Debug, PartialEq)]
pub struct HolderItem {
    pub words: String,
    pub from: f64,
    pub to: f64,
    pub taper: String,
}

/// The shown rows: indices into the library rows and into the document's
/// component rows, and the counts before the search.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct HolderHits {
    pub library: Vec<usize>,
    pub user: Vec<usize>,
    pub library_total: usize,
    pub user_total: usize,
}

impl HolderHits {
    #[must_use]
    pub fn shown(&self) -> usize {
        self.library.len() + self.user.len()
    }

    #[must_use]
    pub fn total(&self) -> usize {
        self.library_total + self.user_total
    }
}

/// The input: the text, the taper pick and the two row lists (kept, so
/// that an address is never reused while it is a key).
type Key = (String, Option<String>, Rc<Vec<HolderItem>>, Rc<Vec<ComponentRow>>);

fn same(a: &Key, b: &Key) -> bool {
    a.0 == b.0 && a.1 == b.1 && Rc::ptr_eq(&a.2, &b.2) && Rc::ptr_eq(&a.3, &b.3)
}

thread_local! {
    static KEPT: RefCell<Option<(Key, Rc<HolderHits>)>> = const { RefCell::new(None) };
    static SEARCHES: Cell<u64> = const { Cell::new(0) };
}

/// The searches run on this thread - a still frame adds none.
#[cfg(test)]
#[must_use]
pub fn holder_searches() -> u64 {
    SEARCHES.with(Cell::get)
}

/// Does `item` match each of `terms`?
fn matches(item: &HolderItem, terms: &[String]) -> bool {
    terms.iter().all(|t| {
        item.words.contains(t.as_str())
            || t.trim_end_matches("mm").parse::<f64>().is_ok_and(|v| v.is_finite() && v >= item.from - 1e-9 && v <= item.to + 1e-9)
    })
}

/// The rows `query` shows: the library rows of the picked `taper` (all
/// when `None`) and the document rows that `user_item` makes an item of.
pub fn holder_hits(
    query: &str,
    taper: Option<&str>,
    library: &Rc<Vec<HolderItem>>,
    user: &Rc<Vec<ComponentRow>>,
    user_item: impl Fn(&ComponentRow) -> Option<HolderItem>,
) -> Rc<HolderHits> {
    let key: Key = (query.to_owned(), taper.map(str::to_owned), Rc::clone(library), Rc::clone(user));
    KEPT.with(|k| {
        let mut k = k.borrow_mut();
        if let Some((kk, hits)) = k.as_ref()
            && same(kk, &key)
        {
            return Rc::clone(hits);
        }
        SEARCHES.with(|s| s.set(s.get().saturating_add(1)));
        let terms: Vec<String> = query.to_lowercase().split_whitespace().map(str::to_owned).collect();
        let in_taper = |i: &HolderItem| taper.is_none_or(|t| i.taper == t);
        let lib: Vec<usize> = library.iter().enumerate().filter(|(_, i)| in_taper(i)).map(|(n, _)| n).collect();
        let users: Vec<(usize, HolderItem)> = user.iter().enumerate().filter_map(|(n, r)| user_item(r).map(|i| (n, i))).collect();
        let hits = Rc::new(HolderHits {
            library_total: lib.len(),
            user_total: users.len(),
            library: lib.into_iter().filter(|n| library.get(*n).is_some_and(|i| matches(i, &terms))).collect(),
            user: users.iter().filter(|(_, i)| matches(i, &terms)).map(|(n, _)| *n).collect(),
        });
        *k = Some((key, Rc::clone(&hits)));
        hits
    })
}
