//! DXF BLOCK definitions and their INSERT references, expanded into world geometry (#1865).
//! Provenance: the BLOCK, ENDBLK and INSERT group codes of Autodesk's published DXF Reference
//! (insertion point, X/Y/Z scale, rotation, column/row array, layer "0" inheritance); our own
//! implementation.
//!
//! A block body is kept as text and read at EOF, at most once, only when an INSERT uses it.
//! Expansion is bounded: nesting by [`MAX_BLOCK_DEPTH`], and every copy and every placed
//! primitive counts against the vertex limit. A cycle, a missing block, an external
//! reference and a zero scale are refused by name.
use super::entities::{common, coordinate, layer_name};
use super::insert_curve::{LayeredCurve, place_curve};
use super::insert_place::{Affine, place};
use super::pairs::{Pairs, Record};
use super::reader::entity_list;
use super::types::{DxfError, LayeredPrimitive, State, error};
use std::collections::BTreeMap;
use std::rc::Rc;

/// The deepest chain of nested INSERTs that is expanded. A top-level INSERT is level 1.
pub const MAX_BLOCK_DEPTH: usize = 8;

/// One INSERT record: a placed copy (or a grid of copies) of a block.
#[derive(Clone, Debug)]
pub struct InsertRef {
    name: String,
    layer: String,
    at: [f64; 3],
    scale: [f64; 3],
    rotation_deg: f64,
    columns: u32,
    rows: u32,
    spacing: [f64; 2],
}

/// Read an INSERT, and its attributes (`ATTRIB` up to `SEQEND`, group 66), which carry no
/// geometry. The reference is kept on `state` and expanded at EOF.
pub(super) fn read(record: Record<'_>, input: &mut Pairs<'_>, state: &mut State) -> Result<(), DxfError> {
    common(record)?;
    let name = record.required(2)?.trim();
    if name.is_empty() || name.len() > 255 {
        return Err(error("INSERT", "block name must contain 1 to 255 bytes"));
    }
    let layer = record.field(8)?.unwrap_or("0");
    layer_name(layer, "INSERT")?;
    let mut at = [0.0; 3];
    for (slot, code) in at.iter_mut().zip([10, 20, 30]) {
        *slot = coordinate(record.number(code, Some(0.0))?, "INSERT")?;
    }
    let mut scale = [1.0; 3];
    for (slot, code) in scale.iter_mut().zip([41, 42, 43]) {
        *slot = record.number(code, Some(1.0))?;
        if !(1e-9..=1e9).contains(&slot.abs()) {
            return Err(error(&format!("INSERT {name}"), "scale factor must be nonzero and within 1e-9 to 1e9"));
        }
    }
    let count = |code: u16| -> Result<u32, DxfError> {
        let n = record.integer(code, 1)?;
        u32::try_from(n)
            .ok()
            .filter(|n| *n >= 1)
            .ok_or_else(|| error(&format!("INSERT {name}"), "array count must be at least 1"))
    };
    let (columns, rows) = (count(70)?, count(71)?);
    let mut spacing = [0.0; 2];
    for (slot, code) in spacing.iter_mut().zip([44, 45]) {
        *slot = coordinate(record.number(code, Some(0.0))?, "INSERT")?;
    }
    state.inserts.push(InsertRef {
        name: name.into(),
        layer: layer.into(),
        at,
        scale,
        rotation_deg: record.number(50, Some(0.0))?.rem_euclid(360.0),
        columns,
        rows,
        spacing,
    });
    if record.integer(66, 0)? == 0 {
        return Ok(());
    }
    loop {
        let next = input
            .record()?
            .ok_or_else(|| error("INSERT", "missing SEQEND after attributes"))?;
        match next.kind {
            "ATTRIB" => {
                state.entity("ATTRIB")?;
                next.validate()?;
            }
            "SEQEND" => {
                state.entity("SEQEND")?;
                return next.validate();
            }
            _ => return Err(error("INSERT", "expected ATTRIB or SEQEND")),
        }
    }
}

struct Definition<'a> {
    name: String,
    base: [f64; 3],
    body: &'a str,
}

/// The block definitions of the BLOCKS section, by upper-case name (DXF names ignore case).
#[derive(Default)]
pub(super) struct Definitions<'a> {
    blocks: BTreeMap<String, Definition<'a>>,
}

impl<'a> Definitions<'a> {
    pub fn len(&self) -> usize {
        self.blocks.len()
    }

    /// Keep the body of the BLOCK `start` up to its ENDBLK, unread.
    pub fn define(&mut self, start: Record<'a>, input: &mut Pairs<'a>) -> Result<(), DxfError> {
        start.validate()?;
        let name = start.required(2)?.trim();
        if name.is_empty() || name.len() > 255 {
            return Err(error("BLOCK", "block name must contain 1 to 255 bytes"));
        }
        let context = format!("BLOCK {name}");
        if start.integer(70, 0)? & (4 | 8) != 0 {
            return Err(error(&context, "external reference (XREF) is unsupported"));
        }
        let mut base = [0.0; 3];
        for (slot, code) in base.iter_mut().zip([10, 20, 30]) {
            *slot = coordinate(start.number(code, Some(0.0))?, &context)?;
        }
        let begin = input.rest;
        let body = loop {
            let before = input.rest;
            let record = input
                .record()?
                .ok_or_else(|| error(&context, "missing ENDBLK"))?;
            match record.kind {
                "ENDBLK" => {
                    record.validate()?;
                    break &begin[..begin.len() - before.len()];
                }
                "BLOCK" | "ENDSEC" | "SECTION" | "EOF" => return Err(error(&context, "missing ENDBLK")),
                _ => {}
            }
        };
        let key = name.to_ascii_uppercase();
        if self.blocks.contains_key(&key) {
            return Err(error(&context, "duplicate block definition"));
        }
        self.blocks.insert(key, Definition { name: name.into(), base, body });
        Ok(())
    }
}

/// A block body as read: its primitives and curves in drawing order, the Z plane they lie in,
/// and its nested INSERTs.
struct Body {
    items: Vec<Item>,
    elevation: Option<f64>,
    inserts: Vec<InsertRef>,
}

/// A SPLINE or ELLIPSE stays a curve until it is placed, then is fitted in world units.
enum Item {
    Primitive(LayeredPrimitive),
    Curve(LayeredCurve),
}

impl Item {
    fn layer(&self) -> &str {
        match self {
            Item::Primitive(p) => &p.layer,
            Item::Curve(c) => &c.layer,
        }
    }
}

/// Merge the primitives and the deferred curves of a body in drawing order.
fn items(primitives: Vec<LayeredPrimitive>, curves: Vec<LayeredCurve>) -> Vec<Item> {
    let mut out = Vec::with_capacity(primitives.len() + curves.len());
    let mut curves = curves.into_iter().peekable();
    for (i, primitive) in primitives.into_iter().enumerate() {
        while let Some(curve) = curves.next_if(|c| c.at <= i) {
            out.push(Item::Curve(curve));
        }
        out.push(Item::Primitive(primitive));
    }
    out.extend(curves.map(Item::Curve));
    out
}

struct Expansion<'d, 'a> {
    defs: &'d Definitions<'a>,
    read: BTreeMap<String, Rc<Body>>,
    stack: Vec<String>,
}

/// Expand every top-level INSERT of `state` into its primitives.
pub(super) fn expand(defs: &Definitions<'_>, state: &mut State) -> Result<(), DxfError> {
    let top = std::mem::take(&mut state.inserts);
    let mut run = Expansion { defs, read: BTreeMap::new(), stack: Vec::new() };
    for insert in &top {
        run.insert(state, insert, Affine::IDENTITY, &insert.layer)?;
    }
    Ok(())
}

impl Expansion<'_, '_> {
    fn insert(&mut self, state: &mut State, insert: &InsertRef, outer: Affine, layer: &str) -> Result<(), DxfError> {
        let context = format!("INSERT {}", insert.name);
        let key = insert.name.to_ascii_uppercase();
        if self.stack.contains(&key) {
            return Err(error(&context, "cyclic block reference"));
        }
        if self.stack.len() >= MAX_BLOCK_DEPTH {
            return Err(error(&context, &format!("blocks nest deeper than {MAX_BLOCK_DEPTH} levels")));
        }
        let defs = self.defs;
        let def = defs
            .blocks
            .get(&key)
            .ok_or_else(|| error(&context, "block is not defined"))?;
        let copies = u64::from(insert.columns) * u64::from(insert.rows);
        if copies > state.limits.max_vertices as u64 {
            return Err(error(&context, &format!("array of {copies} copies exceeds the vertex limit")));
        }
        let base = def.base;
        let body = self.body(state, &key)?;
        self.stack.push(key);
        let (sin, cos) = sin_cos_deg(insert.rotation_deg);
        let [sx, sy, sz] = insert.scale;
        for row in 0..insert.rows {
            for column in 0..insert.columns {
                count(state, &context)?;
                // Array spacing runs along the rotated, unscaled block axes.
                let dx = f64::from(column) * insert.spacing[0];
                let dy = f64::from(row) * insert.spacing[1];
                let local = Affine::insert(
                    [cos * sx, -sin * sy, sin * sx, cos * sy],
                    [insert.at[0] + cos * dx - sin * dy, insert.at[1] + sin * dx + cos * dy, insert.at[2]],
                    sz,
                    base,
                );
                let m = outer.then(&local);
                if let Some(z) = body.elevation {
                    state.elevation("INSERT", coordinate(m.z(z), &context)?)?;
                }
                for item in &body.items {
                    // DXF rule: block geometry on layer "0" takes the layer of its INSERT.
                    let to = if item.layer() == "0" { layer } else { item.layer() };
                    let placed = match item {
                        Item::Primitive(p) => place(state, &m, &p.primitive, &context)?,
                        Item::Curve(c) => place_curve(state, &m, &c.curve, &context)?,
                    };
                    for primitive in placed {
                        count(state, &context)?;
                        state.push(to, primitive);
                    }
                }
                for nested in &body.inserts {
                    let to = if nested.layer == "0" { layer } else { nested.layer.as_str() };
                    self.insert(state, nested, m, to)?;
                }
            }
        }
        self.stack.pop();
        Ok(())
    }

    /// The body of block `key`, read on first use. TEXT and other annotation in a block is
    /// skipped and counted, as at top level, but not kept as an annotation.
    fn body(&mut self, state: &mut State, key: &str) -> Result<Rc<Body>, DxfError> {
        if let Some(body) = self.read.get(key) {
            return Ok(Rc::clone(body));
        }
        let defs = self.defs;
        let def = defs.blocks.get(key).ok_or_else(|| error(key, "block is not defined"))?;
        let primitives = std::mem::take(&mut state.doc.primitives);
        let elevation = state.doc.elevation.take();
        let annotations = std::mem::take(&mut state.doc.annotations);
        let inserts = std::mem::take(&mut state.inserts);
        let curves = std::mem::take(&mut state.curves);
        let defer = std::mem::replace(&mut state.defer_curves, true);
        let result = entity_list(&mut Pairs { rest: def.body }, state, None);
        state.defer_curves = defer;
        let body = Rc::new(Body {
            items: items(
                std::mem::replace(&mut state.doc.primitives, primitives),
                std::mem::replace(&mut state.curves, curves),
            ),
            elevation: std::mem::replace(&mut state.doc.elevation, elevation),
            inserts: std::mem::replace(&mut state.inserts, inserts),
        });
        state.doc.annotations = annotations;
        result.map_err(|e| DxfError(format!("BLOCK {}: {}", def.name, e.0)))?;
        self.read.insert(key.into(), Rc::clone(&body));
        Ok(body)
    }
}

/// One unit of expansion work: a copy or a placed primitive. Bounded by the vertex limit.
fn count(state: &mut State, context: &str) -> Result<(), DxfError> {
    if state.doc.summary.vertices >= state.limits.max_vertices {
        return Err(error(context, "expanded block geometry exceeds the vertex limit"));
    }
    state.doc.summary.vertices += 1;
    Ok(())
}

/// Sine and cosine of an angle in degrees in `[0, 360)`, exact at the quarter turns.
fn sin_cos_deg(deg: f64) -> (f64, f64) {
    if deg == 0.0 {
        (0.0, 1.0)
    } else if deg == 90.0 {
        (1.0, 0.0)
    } else if deg == 180.0 {
        (0.0, -1.0)
    } else if deg == 270.0 {
        (-1.0, 0.0)
    } else {
        deg.to_radians().sin_cos()
    }
}
