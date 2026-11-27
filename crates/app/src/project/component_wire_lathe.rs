//! #3645 / #3643: the wire form of a lathe component's body
//! (`lathe_body` of a `components` row, v84): its facets in the
//! component's own frame, `[[x, y, z]; 3]` each. Our own JSON over our
//! own record; no vendor format was read or consulted.
//!
//! The law: strict. The facets are built again through the checked
//! constructor (`LatheBody::mesh`), so no facet, or a non-finite corner,
//! is refused by name; an unknown key is refused.

use serde::{Deserialize, Serialize};

use camcraft_tools::lathe_assembly::LatheBody;

/// A lathe component's body on file.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct LatheBodyDto {
    pub facets: Vec<[[f64; 3]; 3]>,
}

pub(super) fn lathe_body_dto(b: &LatheBody) -> LatheBodyDto {
    LatheBodyDto { facets: b.facets().to_vec() }
}

pub(super) fn lathe_body(b: &LatheBodyDto) -> Result<LatheBody, String> {
    LatheBody::mesh(b.facets.clone()).map_err(|e| format!("its lathe body: {e}"))
}
