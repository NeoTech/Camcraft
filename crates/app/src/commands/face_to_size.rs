//! `FaceToSize` (#3196): face a block to height from numbers, as ONE
//! undoable step - an optional `SetStock` (a box of the stated size,
//! its top at the stated top Z) and a normal `CreateOperation` of a
//! Face on the setup's stock, from the top Z down to the final Z in
//! passes of the stated depth. The Z values are program Z: measured
//! from the active setup's placed WCS, else from the world origin.
//! Every number is checked at construction; a refusal names it and
//! leaves the world as it was.
//!
//! Provenance: our own design over plain facing practice (top Z, final
//! Z, depth per pass, stepover); the `SetSheetJob` composite shape; no
//! vendor material consulted.

use flecs_ecs::prelude::World;

use crate::commands::{BatchCommand, Command, CommandError, CreateOperation, SetStock};
use crate::model;
use crate::operation::{Height, OpParams, SrcRef};
use crate::schema::Location;
use crate::stock::StockShape;

/// How far the clearance plane stands over the top Z, mm (the seed's).
const CLEARANCE_MM: f64 = 25.0;
/// The slack a top Z may sit under the stock top (a float round-off).
const TOP_SLACK_MM: f64 = 1e-6;
/// The name a face to size takes when the caller gives none.
pub const DEFAULT_NAME: &str = "Face to size";

/// What a face to size asks for, in plain machining terms (mm, mm/min).
#[derive(Clone, Debug, PartialEq)]
pub struct FaceToSizeSpec {
    /// The operation's name; `None` takes the first free [`DEFAULT_NAME`].
    pub name: Option<String>,
    /// The facing tool, by its document name.
    pub tool: String,
    /// A box stock `[length X, width Y, height Z]` with its top at
    /// `top_z`; `None` faces the setup's stock as it stands.
    pub stock: Option<[f64; 3]>,
    /// The Z of the first pass's top, program Z.
    pub top_z: f64,
    /// The finished Z, program Z.
    pub final_z: f64,
    /// Depth per pass.
    pub stepdown: f64,
    /// Radial stepover between rows.
    pub stepover: f64,
    /// Climb (true) or conventional cut.
    pub climb: bool,
    /// Cutting feed.
    pub feed: f64,
    /// Plunge feed.
    pub plunge_feed: f64,
}

/// The stock and the Face as one undo step.
pub struct FaceToSize {
    operation: String,
    stock_set: bool,
    batch: BatchCommand,
}

impl std::fmt::Debug for FaceToSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FaceToSize").field("operation", &self.operation).field("stock_set", &self.stock_set).finish_non_exhaustive()
    }
}

fn invalid(why: impl Into<String>) -> CommandError {
    CommandError::Invalid(why.into())
}

/// The numbers alone, before the world is read.
fn check_numbers(s: &FaceToSizeSpec) -> Result<(), CommandError> {
    let named = [
        ("top Z", s.top_z),
        ("final Z", s.final_z),
        ("depth per pass", s.stepdown),
        ("stepover", s.stepover),
        ("feed", s.feed),
        ("plunge feed", s.plunge_feed),
    ];
    if let Some((what, v)) = named.iter().find(|(_, v)| !v.is_finite()) {
        return Err(invalid(format!("the {what} {v} is not a number")));
    }
    if s.final_z >= s.top_z {
        return Err(invalid(format!("the final Z {} must be below the top Z {}", s.final_z, s.top_z)));
    }
    if s.stepdown <= 0.0 {
        return Err(invalid(format!("the depth per pass {} must be more than 0", s.stepdown)));
    }
    if s.stepover <= 0.0 {
        return Err(invalid(format!("the stepover {} must be more than 0", s.stepover)));
    }
    if s.feed <= 0.0 || s.plunge_feed <= 0.0 {
        return Err(invalid(format!("the feed {} and the plunge feed {} must be more than 0", s.feed, s.plunge_feed)));
    }
    if let Some(size) = s.stock
        && !size.iter().all(|v| v.is_finite() && *v > 0.0)
    {
        return Err(invalid(format!("the stock size {size:?} must be three numbers more than 0")));
    }
    if s.tool.trim().is_empty() {
        return Err(CommandError::Needs("face to size needs a tool - add a facing tool first".into()));
    }
    Ok(())
}

/// The tool must be a document tool at least as wide as the stepover:
/// a wider stepover leaves ribs standing between the rows.
fn check_tool(world: &World, s: &FaceToSizeSpec) -> Result<(), CommandError> {
    let entity = model::find_by_name(world, &s.tool).ok_or_else(|| CommandError::NotFound(s.tool.clone()))?;
    let tool = model::tool_of(world, &entity).ok_or_else(|| invalid(format!("'{}' is not a tool", s.tool)))?;
    if s.stepover > tool.diameter {
        return Err(invalid(format!(
            "the stepover {} is wider than the tool '{}' ({} mm): ribs would stand between the rows",
            s.stepover, s.tool, tool.diameter
        )));
    }
    Ok(())
}

/// The first free name: the caller's, or `Face to size`, `Face to size 2`...
fn op_name(world: &World, asked: Option<&str>) -> String {
    if let Some(n) = asked {
        return n.to_owned();
    }
    (1..)
        .map(|i| if i == 1 { DEFAULT_NAME.to_owned() } else { format!("{DEFAULT_NAME} {i}") })
        .find(|n| model::find_by_name(world, n).is_none())
        .unwrap_or_else(|| DEFAULT_NAME.to_owned())
}

/// The world Z of program Z0 for the active setup: its placed WCS, else 0.
fn program_zero(world: &World, setup: Option<&str>) -> [f64; 3] {
    crate::setup::wcs_entity(world, setup)
        .and_then(|e| model::transform_of(world, &e))
        .map_or([0.0; 3], |l| [l.pos.x, l.pos.y, l.pos.z])
}

/// The Face that steps from `top` down to `bottom` (world Z).
fn face_params(s: &FaceToSizeSpec, top: f64, bottom: f64) -> OpParams {
    OpParams::Face {
        holes: Default::default(),
        tilt: None,
        stepover: s.stepover,
        clearance_z: Height::fixed(top + CLEARANCE_MM),
        cut_z: Height::fixed(bottom),
        top_z: Some(Height::fixed(top)),
        stepdown: Some(s.stepdown),
        feed: s.feed,
        plunge_feed: s.plunge_feed,
        climb: s.climb,
        blend_turns: false,
        blend_radius: None,
        picks: Default::default(),
    }
}

/// The stock the face cuts must hold the final Z, and its top must not
/// stand over the top Z: the first pass would cut more than one depth.
fn check_against_stock(stock_top: f64, stock_bottom: f64, top: f64, bottom: f64, s: &FaceToSizeSpec) -> Result<(), CommandError> {
    if top < stock_top - TOP_SLACK_MM {
        return Err(invalid(format!(
            "the top Z {} is below the stock top (Z{:.3} here): the first pass would cut more than the depth per pass",
            s.top_z,
            stock_top - (top - s.top_z)
        )));
    }
    if bottom <= stock_bottom {
        return Err(invalid(format!(
            "the final Z {} is at or below the stock bottom (Z{:.3} here): the face would cut through the block",
            s.final_z,
            stock_bottom - (bottom - s.final_z)
        )));
    }
    Ok(())
}

impl FaceToSize {
    /// Validate `spec` against the world and build the step. Refuses by
    /// name: a final Z not below the top Z, a depth per pass or stepover
    /// not above 0, a missing tool or one narrower than the stepover, no
    /// stock (and no size), a top Z under the stock top, or a final Z at
    /// or under the stock bottom.
    pub fn new(world: &World, spec: &FaceToSizeSpec) -> Result<Self, CommandError> {
        check_numbers(spec)?;
        check_tool(world, spec)?;
        let setup = crate::setup::active(world);
        let zero = program_zero(world, setup.as_deref());
        let (top, bottom) = (zero[2] + spec.top_z, zero[2] + spec.final_z);
        let name = op_name(world, spec.name.as_deref());
        let params = face_params(spec, top, bottom);
        let stock = match spec.stock {
            Some([dx, dy, dz]) => {
                check_against_stock(top, top - dz, top, bottom, spec)?;
                // Kept where the stock sat in XY; at the WCS when none.
                let (cx, cy) = model::stock_world_aabb_for(world, setup.as_deref())
                    .map_or((zero[0], zero[1]), |b| ((b.min.x + b.max.x) / 2.0, (b.min.y + b.max.y) / 2.0));
                let at = Location { pos: crate::geometry::Point3 { x: cx, y: cy, z: top - dz }, ..Location::default() };
                Some(SetStock::new(world, StockShape::Box { dx, dy, dz }, at)?)
            }
            None => {
                let b = model::stock_world_aabb_for(world, setup.as_deref()).ok_or_else(|| {
                    CommandError::Needs("face to size needs stock - give the stock size, or set up the stock first".into())
                })?;
                check_against_stock(b.max.z, b.min.z, top, bottom, spec)?;
                None
            }
        };
        // The Face is checked against the stock it will cut: the new stock
        // stands in the world while the op is built, then is taken back.
        if let Some(s) = &stock {
            s.apply(world);
        }
        let op = CreateOperation::new(world, &name, &spec.tool, SrcRef::Stock, params);
        if let Some(s) = &stock {
            s.invert(world);
        }
        let op = op?;
        let stock_set = stock.is_some();
        let mut steps: Vec<Box<dyn Command>> = Vec::with_capacity(2);
        if let Some(s) = stock {
            steps.push(Box::new(s));
        }
        steps.push(Box::new(op));
        Ok(Self { operation: name, stock_set, batch: BatchCommand::new("Face to size", steps) })
    }

    /// The operation this step creates.
    #[must_use]
    pub fn operation(&self) -> &str {
        &self.operation
    }

    /// Whether this step sets the stock (else it faces the stock as it stands).
    #[must_use]
    pub fn stock_set(&self) -> bool {
        self.stock_set
    }
}

impl Command for FaceToSize {
    fn apply(&self, world: &World) {
        self.batch.apply(world);
    }

    fn invert(&self, world: &World) {
        self.batch.invert(world);
    }
}
