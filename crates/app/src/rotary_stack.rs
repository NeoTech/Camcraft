//! The stack under a rotary (#821): a 4th axis bolted to a rotary
//! subplate, the subplate pinned to a fixture plate's dowel pair, the
//! plate on the table. Every layer raises the rotary's centerline, so
//! the spindle nose comes nearer to it than the machine file's
//! `nose_to_axis` (the rotary standing on the bare table) says. The
//! index-swing clause (`posting::index_swing`) judges with the
//! EFFECTIVE figure this module gives.
//!
//! The laws:
//! - **Who raises the rotary.** A setup's rotary subplates (the
//!   `rotary_subplate` accessory) and every plate each one stands on,
//!   through its plate location (`plate::mount`) - a subplate on a plate
//!   on a riser plate counts all three. A setup without a subplate has no
//!   stack: it is judged on the machine's own figure, unchanged.
//! - **Every thickness stated.** A built-in plate states its thickness; an
//!   imported plate (its grid detected from holes) does not, nor does a
//!   plate the location names but the document no longer holds - the
//!   rise is then unknown and the swing is refused by name, never judged
//!   on the table figure.
//! - **More than one subplate:** the tallest stack, the nearest any
//!   centerline can come.
//! - **The setup's stated `nose_to_axis` wins** (`SetupDef::nose_to_axis`):
//!   a measured figure for this setup's stack, used as it is.
//!
//! Provenance: our own design (issue #821 and its 2026-11-11 correction:
//! the subplate is a base plate that does not turn with A); plain
//! arithmetic of stacked heights.

use flecs_ecs::prelude::World;

use crate::accessory::{Accessory, subplate};
use crate::fixture;
use crate::model;
use crate::plate::{MountAt, PlateDef, PlateMount};
use crate::setup;

/// The most plates one stack may count before it reads as a cycle.
const MAX_LAYERS: usize = 64;

/// One subplate's stack: its total thickness and its layers, top down.
#[derive(Clone, Debug, PartialEq)]
pub struct Stack {
    pub thickness: f64,
    pub layers: Vec<(String, f64)>,
}

impl Stack {
    /// "'Rotary subplate' 19.050 mm on 'Plate 12 x 12 in' 24.130 mm".
    #[must_use]
    pub fn describe(&self) -> String {
        self.layers.iter().map(|(n, t)| format!("'{n}' {t:.3} mm")).collect::<Vec<_>>().join(" on ")
    }
}

/// Where the setup's rotary centerline is judged from.
#[derive(Clone, Debug, PartialEq)]
pub enum Rise {
    /// No stack and no stated figure: the machine file's `nose_to_axis`.
    Table,
    /// The setup's stated `nose_to_axis`, mm - it wins.
    Stated(f64),
    /// The machine's figure less this stack.
    Stack(Stack),
    /// A stack (or a stated figure) that cannot be used: why, by name.
    Refused(String),
}

/// The setup's own stated `nose_to_axis`, mm (`None`: not stated).
#[must_use]
pub fn stated(world: &World, declared: Option<&str>) -> Option<f64> {
    let name = setup::effective(world, declared)?;
    setup::def_of(world, &name)?.nose_to_axis
}

/// The setup's rise (see the module laws).
#[must_use]
pub fn rise(world: &World, declared: Option<&str>) -> Rise {
    if let Some(v) = stated(world, declared) {
        return if v.is_finite() && v > 0.0 {
            Rise::Stated(v)
        } else {
            Rise::Refused(format!("the setup's stated nose_to_axis {v} is no length over zero"))
        };
    }
    let mut tallest: Option<Stack> = None;
    for (name, e) in fixture::tagged(world, declared) {
        let Some(def) = fixture::def_of(world, &e) else { continue };
        let Some((t, _)) = def.accessory.as_ref().and_then(subplate::figures) else { continue };
        let s = match stack_of(world, &name, t, def.plate.mount.as_ref()) {
            Ok(s) => s,
            Err(why) => return Rise::Refused(why),
        };
        if tallest.as_ref().is_none_or(|b| s.thickness > b.thickness) {
            tallest = Some(s);
        }
    }
    tallest.map_or(Rise::Table, Rise::Stack)
}

/// Subplate `name` (`t` thick) and every plate under it.
fn stack_of(world: &World, name: &str, t: f64, mount: Option<&PlateMount>) -> Result<Stack, String> {
    let unknown = |what: String| format!("rotary subplate '{name}' stands on {what}, so the rotary's rise is unknown");
    if !(t.is_finite() && t > 0.0) {
        return Err(format!("rotary subplate '{name}' states no usable thickness ({t})"));
    }
    let mut layers = vec![(name.to_owned(), t)];
    let mut total = t;
    let mut next = mount.map(|m| m.plate.clone());
    while let Some(p) = next {
        if layers.len() > MAX_LAYERS || layers.iter().any(|(n, _)| *n == p) {
            return Err(unknown(format!("a plate stack that loops back through '{p}'")));
        }
        let def = model::find_by_name(world, &p)
            .and_then(|e| fixture::def_of(world, &e))
            .ok_or_else(|| unknown(format!("plate '{p}', which is not in the document")))?;
        let th = def
            .plate
            .plate
            .as_ref()
            .and_then(|g| g.params.as_ref())
            .map(|g| g.thickness)
            .ok_or_else(|| unknown(format!("'{p}', a plate whose thickness is not stated (an imported plate)")))?;
        if !(th.is_finite() && th > 0.0) {
            return Err(unknown(format!("'{p}', whose thickness {th} is no length over zero")));
        }
        total += th;
        layers.push((p, th));
        next = def.plate.mount.map(|m| m.plate);
    }
    if !total.is_finite() {
        return Err(unknown("a stack whose height is not finite".into()));
    }
    Ok(Stack { thickness: total, layers })
}

/// The pin law of placing `rider` (its accessory `acc`) at `m` on plate
/// `def`: a rotary subplate is located by its two pin bores, so it goes
/// on a dowel pair whose spacing is its bore spacing. Any other fixture
/// passes.
pub fn pins_fit(def: &PlateDef, m: &PlateMount, rider: &str, acc: Option<&Accessory>) -> Result<(), String> {
    let Some((_, spacing)) = acc.and_then(subplate::figures) else {
        return Ok(());
    };
    match &m.at {
        MountAt::Hole(h) => Err(format!(
            "'{rider}' is a rotary subplate, located by its two pin bores - place it on a dowel pair of '{}', not at hole {}",
            m.plate,
            h.trim().to_ascii_uppercase()
        )),
        MountAt::Dowels([a, b]) => {
            let g = def.dowel_lattice().map_err(|e| format!("plate '{}': {e}", m.plate))?;
            let (ax, ay) = g.locate(a).map_err(|e| format!("plate '{}': dowel {e}", m.plate))?;
            let (bx, by) = g.locate(b).map_err(|e| format!("plate '{}': dowel {e}", m.plate))?;
            let d = (bx - ax).hypot(by - ay);
            if (d - spacing).abs() > subplate::SPACING_TOL {
                return Err(format!(
                    "'{rider}' has its pin bores {spacing:.3} mm apart, but {} on '{}' are {d:.3} mm apart",
                    m.at.label(),
                    m.plate
                ));
            }
            Ok(())
        }
    }
}
