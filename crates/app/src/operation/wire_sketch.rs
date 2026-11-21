//! Resolve the sole DXF punch on its stock face for generation, verification and native posting.
use super::{OpParams, SrcRef, WireSide, material::Material};
use crate::{
    geo2d::{Loop, Point2},
    geometry::{Point3, transform_point},
    model,
    schema::Operation,
    setup, sketch,
};
use camcraft_geometry::{dxf::Primitive, wire_round_profile::RoundProfile};
use flecs_ecs::prelude::{Entity, World};
use std::hash::Hasher;

pub(super) struct Placed {
    pub shape: RoundProfile,
    pub ring: Loop,
    z: [f64; 2],
}

impl Placed {
    pub fn material(&self) -> Material {
        Material {
            facets: self.shape.prism(&self.ring, self.z[0], self.z[1]),
            top: self.z[1],
            bodies: Vec::new(),
            excess: 0.0,
        }
    }
}

pub(super) fn placed(world: &World, op: &Operation) -> Result<Option<Placed>, String> {
    let (OpParams::WireContour(w), SrcRef::Entity(name)) = (&op.params, &op.src) else {
        return Ok(None);
    };
    let Some((entity, drawing)) = sketch::find(world, name) else {
        return Ok(None);
    };
    let setup = op.setup.as_deref();
    if !matches!(w.side, WireSide::Punch)
        || w.taper_deg != 0.
        || w.four_axis.is_some()
        || w.tabs.is_some()
        || w.burnout.is_some()
        || !w.skims.is_empty()
        || !w.capture
        || w.start.is_none()
    {
        return Err(
            "The DXF Wire Contour requires a captured vertical punch and a threading hole".into(),
        );
    }
    let stock = setup::stock_entity(world, setup).ok_or("The DXF wire punch needs box stock")?;
    if !matches!(
        model::stock_def_of(world, &stock),
        Some(crate::stock::StockShape::Box { .. })
    ) || !model::model_entities(world).is_empty()
        || !matches!(sketch::placed_in(world, setup).as_slice(), [only] if only == name)
        || model::transform_of(world, &stock)
            .is_some_and(|at| !crate::setup_align::is_identity_rot(at.rot))
        || crate::setup_align::alignment(world, setup).is_some()
    {
        return Err(
            "The DXF wire punch needs one placed drawing on its own axis-aligned box stock".into(),
        );
    }
    if crate::setup::effective(world, setup)
        != crate::setup::effective(world, drawing.setup.as_deref())
    {
        return Err("The wire punch and DXF profile must use the same setup".into());
    }
    let pose =
        sketch::world_pose(world, &entity).ok_or("Cannot place the DXF punch on the stock")?;
    let mut shape = RoundProfile::from_sketch(&drawing.data)?;
    shape.place_centers(|p| {
        let xy = transform_point(Point3::new(p.x, p.y, 0.), pose.pos, pose.rot);
        Point2::new(xy.x, xy.y)
    });
    let stock =
        model::stock_world_aabb_for(world, setup).ok_or("Cannot resolve the DXF stock bounds")?;
    let hs = super::resolve_params(world, setup, Some(&op.src), &op.params)?;
    if hs.top != Some(stock.max.z)
        || hs.bottom != Some(stock.min.z)
        || (pose.pos.z - stock.max.z).abs() > 1e-8
    {
        return Err("The DXF punch must pass through the stock from its top to its bottom".into());
    }
    let ring = shape.preview()?;
    if ring
        .points()
        .iter()
        .any(|p| p.x < stock.min.x || p.x > stock.max.x || p.y < stock.min.y || p.y > stock.max.y)
    {
        return Err("The entire placed DXF punch must fit in the stock".into());
    }
    Ok(Some(Placed {
        shape,
        ring,
        z: [stock.min.z, stock.max.z],
    }))
}

pub(super) fn stamp(world: &World, entity: &Entity, hash: &mut impl Hasher) -> Option<()> {
    let sketch = sketch::sketch_of(world, entity)?;
    let pose = crate::sketch::world_pose(world, entity)?;
    hash.write(b"through-dxf-punch-v1");
    let number = |h: &mut _, v: f64| Hasher::write_u64(h, v.to_bits());
    for v in [pose.pos.x, pose.pos.y, pose.pos.z]
        .into_iter()
        .chain(pose.rot)
    {
        number(hash, v);
    }
    number(hash, sketch.data.join_tolerance());
    hash.write_usize(sketch.data.skipped_total());
    for shape in sketch.data.primitives() {
        hash.write_usize(shape.layer.len());
        hash.write(shape.layer.as_bytes());
        match shape.primitive {
            Primitive::Line { start, end } => {
                hash.write_u8(0);
                for v in [start.x, start.y, end.x, end.y] {
                    number(hash, v);
                }
            }
            Primitive::Arc {
                center,
                radius,
                start_angle,
                sweep_angle,
            } => {
                hash.write_u8(1);
                for v in [center.x, center.y, radius, start_angle, sweep_angle] {
                    number(hash, v);
                }
            }
            Primitive::Circle { center, radius } => {
                hash.write_u8(2);
                for v in [center.x, center.y, radius] {
                    number(hash, v);
                }
            }
        }
    }
    Some(())
}
