//! The program plan the stock sim walks: each operation of the program,
//! in posting order, resolved to its cached moves and the atom its tool
//! sweeps (`Planned`). The column verdict and the field eye read the one
//! plan. Split from `stock_sim.rs` (#2528); structure only.
//! Provenance: our own stock-simulation design (see `stock_sim`).

use super::*;

/// One operation of the program, resolved for the sim: its cached
/// moves and the atom its tool sweeps.
pub(crate) struct Planned {
    /// The operation's name - the sim timeline's key for it.
    pub name: String,
    pub moves: Vec<crate::cl::Move>,
    pub radius: f64,
    pub ball: bool,
    pub cone: Option<f64>,
    /// The bull nose's corner radius (M13.1), `bull_corner`'s law.
    pub corner: Option<f64>,
    pub setup: Option<String>,
    /// A rotary op's fitted axis (M13.1): the eye chords its turns
    /// about it (`rotary::simulate_rotary_slice`); the column verdict
    /// refuses the program by name. `None` for every other kind.
    pub rotary: Option<crate::verify::RotaryAxis>,
    /// A Turn-family op's nose and spindle (#19): the eye revolves
    /// its moves about the axis (`lathe::lathe_plan`); the column
    /// verdict refuses the program by name. `None` for every mill op.
    pub lathe: Option<LathePlan>,
    /// The edge breaks the op was asked to cut (#303); empty for every
    /// kind but the chamfer and the deburr.
    pub breaks: Vec<crate::cl::EdgeBreak>,
    /// #2435: the breaks are a 2D Chamfer's, whose tip the law puts on
    /// a wall - credited only where it stands on one (`on_walls`).
    pub walled: bool,
    /// #2596: a 2D Chamfer with no picks (its source the whole model):
    /// on a model that carries a modelled bevel, none of its breaks are
    /// credited (`stock_sim_breaks::program_breaks`).
    pub auto_edges: bool,
    /// The mark budget and target part(s) the op declares (#1595, #1627,
    /// `operation::mark_law`): `Some` only for an Engrave or Trace.
    pub mark: Option<crate::operation::mark_law::SimMark>,
    /// A sinker electrode's atom (#541, #1152). The column verdict cuts
    /// it; the field eye has no electrode atom and refuses the program
    /// by name. `None` for every other tool.
    pub electrode: Option<held::ElectrodeAtom>,
    /// #2441: a twist drill's cone carries a cylindrical flute above
    /// `r * cot`, unlike a truncated chamfer `Taper`.
    pub drill: bool,
    /// An undercut cutter's pass (#1444): the head and neck solids and
    /// the moves they cut. `moves` above is then EMPTY: an undercut
    /// clears no column, so every column reader (the program verdict,
    /// the carried stock, the rest grids) cuts nothing for it, and only
    /// the field eye removes the true profile
    /// (`Stock::simulate_undercut`). `None` for every other tool.
    pub undercut: Option<UndercutPlan>,
    /// Declared inside-corner relief corridors computed live from operation geometry (#2400).
    pub corner_reliefs: Vec<crate::verify::CornerReliefZone>,
}

/// An undercut op's cut, for the field eye ([`Planned::undercut`]).
pub(crate) struct UndercutPlan {
    pub cutter: camcraft_tools::undercut::Undercut,
    /// The neck's length above the head, mm.
    pub neck_len: f64,
    pub moves: Vec<crate::cl::Move>,
}

/// The corner radius a tool's atom carries (M13.1): Some only for a
/// bull nose with a real corner, clamped to the tool radius (the
/// `Tool::is_sane` bound, restated so the atom can never be handed a
/// corner wider than its tool). A zero corner IS the flat end mill,
/// cut by the disc exactly; a non-finite one refuses by name.
pub(crate) fn bull_corner(tool: &crate::tools::Tool) -> Result<Option<f64>, String> {
    if tool.kind != crate::tools::ToolKind::BullNose {
        return Ok(None);
    }
    let (c, r) = (tool.corner_r, tool.diameter * 0.5);
    if !(c.is_finite() && r.is_finite() && c >= 0.0 && r > 0.0) {
        return Err(format!("bull nose carries no sane corner (corner radius {c}, radius {r})"));
    }
    Ok((c > 0.0).then_some(c.min(r)))
}

/// The program in posting order, each op resolved to its tool's atom
/// - the one plan both the column verdict and the field eye walk.
pub(crate) fn plan_program(world: &flecs_ecs::prelude::World) -> Result<Vec<Planned>, String> {
    plan_ops(world, &crate::posting::program_ops(world)?)
}

/// The playback's plan (#2280): [`plan_program`] up to the first op
/// with no toolpath, and that op and every op after it, in program
/// order (empty when every op has a toolpath). The verdict and the
/// post never read this: they keep `plan_program`, which refuses.
pub(crate) fn plan_program_until_ungenerated(
    world: &flecs_ecs::prelude::World,
) -> Result<(Vec<Planned>, crate::posting::OpList), String> {
    let (ops, rest) = crate::posting::program_ops_until_ungenerated(world)?;
    Ok((plan_ops(world, &ops)?, rest))
}

fn plan_ops(world: &flecs_ecs::prelude::World, ops: &[(String, flecs_ecs::prelude::Entity)]) -> Result<Vec<Planned>, String> {
    use crate::model;

    // Tools and cached paths, resolved once, in program order. The
    // atom follows the tool kind (stage A's law): a ball nose sweeps
    // the exact sphere; a chamfer mill sweeps its exact cone (M2.4 B
    // - the disc would phantom-gouge every chamfer wall against the
    // full radius at the corner depth); a bull nose sweeps its exact
    // corner-rounded solid (M13.1 - the disc it rode before removed
    // the fillet's ring the real tool leaves); everything whose cut
    // IS its cylinder - end mills, drills, bores - sweeps the disc,
    // and a tap sweeps the disc of its thread's core (#266,
    // `tool_law::cutting_radius`).
    let mut planned: Vec<Planned> = Vec::with_capacity(ops.len());
    for (name, e) in ops {
        let op = model::operation_of(world, e)
            .ok_or_else(|| format!("{name}: lost its operation"))?;
        if wire::skips(&op) || waterjet::skips(&op) {
            continue;
        }
        // The rotary clause (M13.1): between two stations a rotary
        // move turns the WORK, so the tool sweeps an annular sector
        // about the fitted axis - a torus-family solid no stock engine
        // has in closed form. The field arm chords the turn
        // (`Stock::simulate_rotary`, the `rotary_chords` pre-pass:
        // pose lerped per sub-chord at a stated sag) with parity
        // against the radial law on the sim's bar and on the app's
        // fixtures (`rotary_field_parity.rs`), so the plan carries the
        // fitted axis for the eye; a kind whose parity or cost is not
        // proved refuses by name here (`rotary::rotary_kind_lift`),
        // and the column VERDICT refuses every rotary program by name
        // in `simulate_document` (`ROTARY_VERDICT_REFUSAL`).
        let rotary = rotary::rotary_plan(world, name, &op)?;
        let tool = model::find_by_name(world, &op.tool)
            .and_then(|t| model::tool_of(world, &t))
            .ok_or_else(|| format!("{name}: tool {} is not in the library", op.tool))?;
        // A touch probe removes no stock (#108): its stylus only ever
        // meets the part tangent, and the protected-move law in
        // `operation::probe_op` already convicted any other meeting -
        // so the probe op carves nothing and stays out of the plan.
        // A plasma torch (#670) cuts THROUGH the plate along a kerf the
        // dexel column model has no atom for: the plasma verdicts (cut
        // order, kerf crossings, the through-cut) are #672's laws, and
        // the generator certified that no lit move's kerf reaches the
        // part - so the torch carves nothing here, said not silenced.
        if matches!(tool.kind, crate::tools::ToolKind::Probe | crate::tools::ToolKind::Torch | crate::tools::ToolKind::Wire) {
            continue;
        }
        let cache = model::toolpath_of(world, e)
            .ok_or_else(|| format!("{name}: lost its toolpath"))?;
        // The cone's slope, cot of the half-angle from the axis -
        // the number the dexel / drill cone reads directly, so the sim and
        // the congruence door in `generate` see the same steel (#2441).
        let (cone, drill) = match tool.kind {
            crate::tools::ToolKind::Taper => {
                let tan_half = (tool.tip_angle.to_radians() * 0.5).tan();
                if tan_half.is_finite() && tan_half > 0.0 && tool.tip_angle > 0.0 && tool.tip_angle < 180.0 {
                    (Some(1.0 / tan_half), false)
                } else {
                    return Err(format!(
                        "{name}: chamfer mill {} carries no sane cone (tip angle {})",
                        op.tool, tool.tip_angle
                    ));
                }
            }
            crate::tools::ToolKind::Drill => {
                if (tool.tip_angle - 180.0).abs() <= 1e-9 {
                    (None, false)
                } else {
                    let tan_half = (tool.tip_angle.to_radians() * 0.5).tan();
                    if tan_half.is_finite() && tan_half > 0.0 && tool.tip_angle > 0.0 && tool.tip_angle < 180.0 {
                        (Some(1.0 / tan_half), true)
                    } else {
                        return Err(format!(
                            "{name}: drill {} carries no sane point cone (tip angle {})",
                            op.tool, tool.tip_angle
                        ));
                    }
                }
            }
            _ => (None, false),
        };
        let corner = bull_corner(&tool).map_err(|e| format!("{name}: {e}"))?;
        // The revolved clause (#19, was M5.2 G.2's refusal): a Turn-
        // family path lives in the half-plane of its fitted axis, so
        // the plan carries the nose and that axis for the field eye's
        // revolved stadium, a blade its own footprint (`lathe`, #21).
        let lathe = lathe::lathe_plan(world, name, &op, &tool, &cache.path.moves)?;
        // A turning insert's cutting feature is its nose (a blade's,
        // half its band - `LathePlan::feature`): the display cell law
        // (`auto_cell`) reads it, not the IC. Every other tool sweeps
        // its cutting radius - a tap its thread's core (#266, #985).
        // A sinker electrode (#541) sweeps its rectangular face - or its
        // outline (#1152) - grown by the condition's spark gap; its
        // radius is the least half extent, which only the cell law reads.
        let electrode = match &op.params {
            crate::operation::OpParams::SinkerBurn(sb) if tool.kind == crate::tools::ToolKind::Electrode => {
                let gap = crate::operation::sinker_op::condition_gap(world, op.setup.as_deref(), sb)
                    .map_err(|e| format!("{name}: {e}"))?;
                Some(held::ElectrodeAtom::of(&tool, gap))
            }
            // #545: the side burn sweeps the same box atom along its
            // lateral move.
            crate::operation::OpParams::SinkerSideBurn(sb) if tool.kind == crate::tools::ToolKind::Electrode => {
                let gap = crate::operation::sinker_side_op::side_gap(world, op.setup.as_deref(), sb)
                    .map_err(|e| format!("{name}: {e}"))?;
                Some(held::ElectrodeAtom::of(&tool, gap))
            }
            _ => None,
        };
        let radius = match (lathe, &electrode) {
            (Some(plan), _) => plan.feature(),
            (None, Some(e)) => e.half[0].min(e.half[1]),
            (None, None) => crate::operation::thread_op::sim_radius(&tool, &op.params).map_err(|e| format!("{name}: {e}"))?,
        };
        // #546: a wear-compensated burn is judged as the worn electrode burns it.
        let moves = crate::operation::sinker_wear_op::judged(world, &op, electrode.as_ref().map(|e| e.gap), cache.path.moves)
            .map_err(|e| format!("{name}: {e}"))?;
        // #1444: an undercut cutter's moves ride its own plan.
        let undercut = if camcraft_tools::undercut::is_kind(tool.kind) {
            let cutter = camcraft_tools::undercut::Undercut::of(&tool).ok_or_else(|| {
                format!(
                    "{name}: {}",
                    camcraft_tools::undercut::refusal(&tool).unwrap_or_else(|| "its tool reads as no undercut cutter".into())
                )
            })?;
            Some(UndercutPlan { cutter, neck_len: tool.shank_len, moves: moves.clone() })
        } else {
            None
        };
        let moves = if undercut.is_some() { Vec::new() } else { moves };
        let corner_reliefs = crate::operation::corner_relief::zones_for_op(world, &op, name);
        planned.push(Planned {
            name: name.clone(),
            moves,
            // #529: only what the op itself declared, never wider. #2308:
            // and never a stock edge's bevel over a part.
            breaks: if breaks::breaks_stock_edge(world, &op) { Vec::new() } else { breaks::bounded(name, &op.params, cache.path.breaks) },
            walled: matches!(op.params, crate::operation::OpParams::Chamfer2D { .. }),
            auto_edges: breaks::auto_edges(&op),
            mark: crate::operation::mark_law::sim_mark_of(world, &op, crate::operation::mark_law::cutter_depth_limit(&op.params, &tool)),
            radius,
            ball: tool.kind == crate::tools::ToolKind::BallNose,
            cone,
            corner,
            setup: op.setup,
            rotary,
            lathe,
            electrode,
            drill,
            undercut,
            corner_reliefs,
        });
    }
    Ok(planned)
}
