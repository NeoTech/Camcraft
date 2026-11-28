//! #3643: an STL or OBJ file read as the facets of one tool component,
//! inside the facet budget of a component body. STL per the public
//! StereoLithography interface layout (ASCII `solid` / `facet` /
//! `vertex` records; binary: an 80-byte header, a little-endian u32
//! count, 50 bytes per facet); OBJ per the public Wavefront OBJ notes
//! (`v x y z`, `f` with 1-based or negative indices and `v/vt/vn`
//! words, polygons split as a fan). Our own reader; no vendor source or
//! file consulted.
//!
//! The law: a file is read whole or refused by name - empty, too big,
//! over the facet budget, not text where text is needed, a syntax
//! error with its line, a non-finite number, an index off the vertex
//! list, or no facets. Nothing here panics on any input.

/// One facet: three corners in mm.
pub type Tri = [[f64; 3]; 3];

/// The most facets one component body may have. A body past it is
/// refused by name (#1041 is the open work to mesh it coarser instead).
pub const MAX_BODY_FACETS: usize = 1_000_000;
/// The largest component file read (bytes): an ASCII STL at the facet
/// budget is about 250 MB.
pub const MAX_BODY_FILE_BYTES: usize = 512 << 20;
/// The most vertices an OBJ body may list.
pub const MAX_BODY_VERTICES: usize = 3 * MAX_BODY_FACETS;

/// Why a mesh file is no component body.
#[derive(Clone, Debug, PartialEq)]
pub enum MeshFileError {
    Empty,
    TooBig { bytes: usize },
    OverBudget { facets: usize },
    TooManyVertices,
    NotText,
    BinaryLength { facets: u64, bytes: usize },
    Syntax { line: usize, why: String },
    NonFinite { line: usize },
    BadIndex { line: usize, word: String },
    NoFacets,
}

impl core::fmt::Display for MeshFileError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Empty => write!(f, "the file is empty"),
            Self::TooBig { bytes } => {
                write!(f, "the file has {bytes} bytes, more than the {MAX_BODY_FILE_BYTES} bytes of a component body")
            }
            Self::OverBudget { facets } => write!(
                f,
                "the body has {facets} facets or more, over the budget of {MAX_BODY_FACETS} facets for one tool \
                 component (#1041: a coarser mesh is not made yet) - export it again with fewer facets"
            ),
            Self::TooManyVertices => {
                write!(f, "the file lists more than {MAX_BODY_VERTICES} vertices, over the budget of a component body")
            }
            Self::NotText => write!(f, "the file is not text (UTF-8)"),
            Self::BinaryLength { facets, bytes } => write!(
                f,
                "the binary STL states {facets} facets, but holds {bytes} bytes (84 + 50 per facet) - the file is cut short or is no STL"
            ),
            Self::Syntax { line, why } => write!(f, "line {line}: {why}"),
            Self::NonFinite { line } => write!(f, "line {line}: a coordinate is not a finite number"),
            Self::BadIndex { line, word } => write!(f, "line {line}: '{word}' is no vertex of the file"),
            Self::NoFacets => write!(f, "the file has no facets"),
        }
    }
}

impl std::error::Error for MeshFileError {}

fn check_size(data: &[u8]) -> Result<(), MeshFileError> {
    if data.is_empty() {
        return Err(MeshFileError::Empty);
    }
    if data.len() > MAX_BODY_FILE_BYTES {
        return Err(MeshFileError::TooBig { bytes: data.len() });
    }
    Ok(())
}

/// The facets of an ASCII or binary STL file.
pub fn read_stl(data: &[u8]) -> Result<Vec<Tri>, MeshFileError> {
    check_size(data)?;
    // A binary header may start with "solid" too: an ASCII STL is text
    // with no NUL byte, and one with no facet record is tried as binary.
    let text_start = data.iter().position(|b| !b.is_ascii_whitespace()).unwrap_or(data.len());
    let solid = data[text_start..].starts_with(b"solid");
    if solid
        && !data.contains(&0)
        && let Ok(text) = std::str::from_utf8(data)
    {
        match read_ascii(text) {
            Err(MeshFileError::NoFacets) => {}
            other => return other,
        }
    }
    let count = data.get(80..84).map(|b| u64::from(u32::from_le_bytes([b[0], b[1], b[2], b[3]])));
    if let Some(n) = count
        && 84 + 50 * n == data.len() as u64
    {
        return read_binary(data, n);
    }
    match count {
        _ if solid && !data.contains(&0) && std::str::from_utf8(data).is_ok() => Err(MeshFileError::NoFacets),
        Some(facets) => Err(MeshFileError::BinaryLength { facets, bytes: data.len() }),
        None if solid => Err(MeshFileError::NotText),
        None => Err(MeshFileError::Syntax { line: 1, why: "an STL starts with 'solid' or an 84-byte binary header".into() }),
    }
}

fn read_binary(data: &[u8], n: u64) -> Result<Vec<Tri>, MeshFileError> {
    if n == 0 {
        return Err(MeshFileError::NoFacets);
    }
    if n > MAX_BODY_FACETS as u64 {
        return Err(MeshFileError::OverBudget { facets: usize::try_from(n).unwrap_or(usize::MAX) });
    }
    let mut tris = Vec::new();
    // The length was checked equal to 84 + 50 n, so every record is whole.
    let (records, _) = data[84..].as_chunks::<50>();
    for (k, rec) in records.iter().enumerate() {
        let f = |i: usize| {
            let at = 12 + 4 * i;
            f64::from(f32::from_le_bytes([rec[at], rec[at + 1], rec[at + 2], rec[at + 3]]))
        };
        let t: Tri = core::array::from_fn(|v| core::array::from_fn(|c| f(3 * v + c)));
        if !t.iter().flatten().all(|x| x.is_finite()) {
            // Binary records have no lines; name the facet as line k + 1.
            return Err(MeshFileError::NonFinite { line: k + 1 });
        }
        tris.push(t);
    }
    Ok(tris)
}

fn read_ascii(text: &str) -> Result<Vec<Tri>, MeshFileError> {
    let mut tris = Vec::new();
    let mut corners: Vec<[f64; 3]> = Vec::with_capacity(3);
    let mut in_facet = false;
    let syntax = |line: usize, why: &str| MeshFileError::Syntax { line, why: why.into() };
    for (i, row) in text.lines().enumerate() {
        let line = i + 1;
        let mut words = row.split_whitespace();
        match words.next() {
            Some("facet") => {
                if in_facet {
                    return Err(syntax(line, "a facet starts inside a facet"));
                }
                in_facet = true;
                corners.clear();
            }
            Some("vertex") => {
                if !in_facet {
                    return Err(syntax(line, "a vertex stands outside a facet"));
                }
                if corners.len() == 3 {
                    return Err(syntax(line, "a facet has more than 3 vertices"));
                }
                corners.push(point(&mut words, line)?);
            }
            Some("endfacet") => {
                if !in_facet {
                    return Err(syntax(line, "'endfacet' without 'facet'"));
                }
                let [a, b, c] = corners[..] else {
                    return Err(syntax(line, "a facet needs exactly 3 vertices"));
                };
                if tris.len() >= MAX_BODY_FACETS {
                    return Err(MeshFileError::OverBudget { facets: tris.len() + 1 });
                }
                tris.push([a, b, c]);
                in_facet = false;
                corners.clear();
            }
            _ => {}
        }
    }
    if in_facet {
        return Err(syntax(text.lines().count(), "the file ends inside a facet"));
    }
    if tris.is_empty() {
        return Err(MeshFileError::NoFacets);
    }
    Ok(tris)
}

fn point<'a>(words: &mut impl Iterator<Item = &'a str>, line: usize) -> Result<[f64; 3], MeshFileError> {
    let mut p = [0.0; 3];
    for c in &mut p {
        let w = words.next().ok_or_else(|| MeshFileError::Syntax { line, why: "a point needs 3 numbers".into() })?;
        let v: f64 =
            w.parse().map_err(|_| MeshFileError::Syntax { line, why: format!("'{w}' is not a number") })?;
        if !v.is_finite() {
            return Err(MeshFileError::NonFinite { line });
        }
        *c = v;
    }
    Ok(p)
}

/// The facets of an OBJ file: every `f` record, a polygon split as a
/// fan from its first corner.
pub fn read_obj(data: &[u8]) -> Result<Vec<Tri>, MeshFileError> {
    check_size(data)?;
    let text = std::str::from_utf8(data).map_err(|_| MeshFileError::NotText)?;
    let mut verts: Vec<[f64; 3]> = Vec::new();
    let mut tris: Vec<Tri> = Vec::new();
    for (i, row) in text.lines().enumerate() {
        let line = i + 1;
        let mut words = row.split_whitespace();
        match words.next() {
            Some("v") => {
                if verts.len() >= MAX_BODY_VERTICES {
                    return Err(MeshFileError::TooManyVertices);
                }
                verts.push(point(&mut words, line)?);
            }
            Some("f") => {
                let corners: Vec<[f64; 3]> =
                    words.map(|w| vertex(w, &verts, line)).collect::<Result<_, _>>()?;
                if corners.len() < 3 {
                    return Err(MeshFileError::Syntax { line, why: "a face needs at least 3 corners".into() });
                }
                for k in 1..corners.len() - 1 {
                    if tris.len() >= MAX_BODY_FACETS {
                        return Err(MeshFileError::OverBudget { facets: tris.len() + 1 });
                    }
                    tris.push([corners[0], corners[k], corners[k + 1]]);
                }
            }
            _ => {}
        }
    }
    if tris.is_empty() {
        return Err(MeshFileError::NoFacets);
    }
    Ok(tris)
}

/// The vertex an `f` word names: `i`, `i/t`, `i//n` or `i/t/n`, with a
/// 1-based index or a negative one counted back from the last vertex.
fn vertex(word: &str, verts: &[[f64; 3]], line: usize) -> Result<[f64; 3], MeshFileError> {
    let bad = || MeshFileError::BadIndex { line, word: word.to_owned() };
    let head = word.split('/').next().unwrap_or_default();
    let i: i64 = head.parse().map_err(|_| bad())?;
    let n = i64::try_from(verts.len()).map_err(|_| bad())?;
    let at = if (1..=n).contains(&i) {
        i - 1
    } else if (-n..0).contains(&i) {
        n + i
    } else {
        return Err(bad());
    };
    usize::try_from(at).ok().and_then(|k| verts.get(k)).copied().ok_or_else(bad)
}
