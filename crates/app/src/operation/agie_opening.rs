//! The placed line and arc walls of one selected DXF opening in the setup work frame, and the
//! check that no other drawing primitive touches or enters it. Provenance: our own glue over
//! CamCraft's selected-chain door (#3180) and analytic chain clearance (#3219).

use std::collections::HashSet;

use camcraft_geometry::dxf::Primitive;
use camcraft_geometry::sketch_chains::{ChainClearance, ChainError};
use camcraft_post::agie_setup::InspectionSegment;

use super::wire_sketch::SelectedWireChain;
use crate::geo2d::Point2;
use crate::geometry::{Point3, quat_conjugate, quat_rotate_point, transform_point};

fn exact_repeat(a: &Primitive, b: &Primitive) -> bool {
    if a == b {
        return true;
    }
    match (a, b) {
        (Primitive::Line { start: a, end: b }, Primitive::Line { start: c, end: d }) => {
            a == d && b == c
        }
        _ => false,
    }
}

impl SelectedWireChain {
    /// A drawing point in the setup work frame whose origin is world XY `origin`.
    pub(super) fn placed(&self, p: Point2, origin: [f64; 2]) -> Point2 {
        let q = transform_point(Point3::new(p.x, p.y, 0.0), self.pose.pos, self.pose.rot);
        Point2::new(q.x - origin[0], q.y - origin[1])
    }

    /// A world XY point in drawing coordinates.
    pub(super) fn local(&self, p: Point2) -> Point2 {
        let q = quat_rotate_point(
            quat_conjugate(self.pose.rot),
            Point3::new(p.x - self.pose.pos.x, p.y - self.pose.pos.y, 0.0),
        );
        Point2::new(q.x, q.y)
    }

    /// The oriented walls of the selected chain in the setup work frame. A
    /// circle is two half arcs from its start point. A joint gap is refused.
    pub(super) fn opening_spans(&self, origin: [f64; 2]) -> Result<Vec<InspectionSegment>, String> {
        let ring = &self.chain;
        ring.nesting.as_ref().map_err(|e| e.to_string())?;
        if ring.max_gap > 1e-8 {
            return Err("the selected opening has an unresolved joint gap".into());
        }
        let place = |p: Point2| self.placed(p, origin);
        let mut spans = Vec::new();
        for seg in &ring.segments {
            let (a, b) = (place(seg.start), place(seg.end));
            let arc = match seg.curve {
                Primitive::Line { .. } => None,
                Primitive::Arc { center, sweep_angle, .. } => Some((place(center), sweep_angle)),
                Primitive::Circle { center, radius } => {
                    let mid = place(Point2::new(center.x - radius, center.y));
                    let half = Some((place(center), std::f64::consts::PI));
                    spans.push(InspectionSegment { start: a, end: mid, arc: half });
                    spans.push(InspectionSegment { start: mid, end: a, arc: half });
                    continue;
                }
            };
            spans.push(InspectionSegment { start: a, end: b, arc });
        }
        Ok(spans)
    }

    /// Refuse every unselected primitive that touches or enters the selected
    /// loop. A retraced line on the same layer is the selection only when the
    /// chainer names its original and both ends match. `extra` sees each
    /// remaining primitive with its index.
    pub(super) fn check_opening_clear(
        &self,
        clearance: &mut ChainClearance,
        mut extra: impl FnMut(usize, &Primitive) -> Result<(), String>,
    ) -> Result<(), String> {
        let data = self.data.primitives();
        let owned: HashSet<_> = self.chain.segments.iter().map(|s| s.primitive).collect();
        let mut used = owned.clone();
        for problem in &self.data.chains().rejected {
            if let ChainError::DuplicatePrimitive { primitive, of } = problem.error
                && owned.contains(&of)
                && let (Some(a), Some(b)) = (data.get(primitive), data.get(of))
                && a.layer == b.layer
                && exact_repeat(&a.primitive, &b.primitive)
            {
                used.insert(primitive);
            }
        }
        for (i, item) in data.iter().enumerate().filter(|(i, _)| !used.contains(i)) {
            if clearance.obstructs(i, &item.primitive, 1e-8).map_err(|e| e.to_string())? {
                return Err(format!(
                    "unselected DXF primitive {} touches or enters the source opening",
                    i + 1
                ));
            }
            extra(i, &item.primitive)?;
        }
        Ok(())
    }
}
