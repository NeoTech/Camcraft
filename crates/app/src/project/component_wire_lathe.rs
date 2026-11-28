//! #3645 / #3643: the wire form of a lathe component's body
//! (`lathe_body` of a `components` row, v84): its facets in the
//! component's own frame, `[[x, y, z]; 3]` each. Our own JSON over our
//! own record; no vendor format was read or consulted.
//!
//! The law: strict. The facets are built again through the checked
//! constructor (`LatheBody::mesh`), so no facet, or a non-finite corner,
//! is refused by name; an unknown key is refused. A meshed body's pad
//! (#3643: the steel up to `pad` mm outside the facets) is written when
//! it is above 0 and read back through the checked `LatheBody::padded`,
//! so a save and an open never drop it.

use serde::{Deserialize, Serialize};

use camcraft_tools::lathe_assembly::LatheBody;

/// A lathe component's body on file.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct LatheBodyDto {
    pub facets: Vec<[[f64; 3]; 3]>,
    #[serde(default, skip_serializing_if = "no_pad")]
    pub pad: f64,
}

fn no_pad(p: &f64) -> bool {
    *p <= 0.0
}

pub(super) fn lathe_body_dto(b: &LatheBody) -> LatheBodyDto {
    LatheBodyDto { facets: b.facets().to_vec(), pad: b.pad() }
}

pub(super) fn lathe_body(b: &LatheBodyDto) -> Result<LatheBody, String> {
    LatheBody::mesh(b.facets.clone()).and_then(|m| m.padded(b.pad)).map_err(|e| format!("its lathe body: {e}"))
}
