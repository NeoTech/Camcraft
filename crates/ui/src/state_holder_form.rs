//! #4338 (U12 of #3663): the shape of a document holder as the holder form
//! edits it (`holder_library::HolderShape::of`), read when the row
//! changes and kept: the form compares its draft with the row's shape on
//! each frame, and a still frame reads no shape again (`shape_reads`).
//! Provenance: our own cache over our own holder shape; no HSMWorks
//! material involved.

use std::cell::{Cell, RefCell};

use camcraft_app::component::{ADAPTIVE_PORT, ComponentKind};
use camcraft_app::component_geometry::ComponentGeometry;
use camcraft_app::holder_library::HolderShape;
use camcraft_app::schema::DocumentComponent;

use super::{ComponentGroup, group_of};

type Kept = Option<((ComponentKind, ComponentGeometry), Option<HolderShape>)>;

thread_local! {
    static KEPT: RefCell<Kept> = const { RefCell::new(None) };
    static READS: Cell<u64> = const { Cell::new(0) };
}

/// The shape reads on this thread - a still frame adds none.
#[cfg(test)]
#[must_use]
pub fn shape_reads() -> u64 {
    READS.with(Cell::get)
}

/// The shape of holder `row`, when the holder form can edit it: a spindle
/// holder whose outline is the holder library's outline of a shape.
#[must_use]
pub fn shape_of(row: &DocumentComponent) -> Option<HolderShape> {
    if group_of(&row.component) != ComponentGroup::Holders {
        return None;
    }
    KEPT.with(|k| {
        let mut k = k.borrow_mut();
        if let Some(((kind, geometry), shape)) = k.as_ref()
            && kind == row.component.kind()
            && *geometry == row.geometry
        {
            return *shape;
        }
        READS.with(|r| r.set(r.get().saturating_add(1)));
        let shape = HolderShape::of(row.component.kind(), &row.geometry, ADAPTIVE_PORT);
        *k = Some(((row.component.kind().clone(), row.geometry.clone()), shape));
        shape
    })
}
