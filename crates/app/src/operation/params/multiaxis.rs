//! The multi-axis arms of `OpParams::stamp_with` (#1849 split): the
//! 5-axis, rotary and swarf kinds. Provenance: moved verbatim from
//! `params.rs`; our own digest law.

use super::*;
use super::digest::{self, w};

impl OpParams {
    /// The digest bytes of the 5-axis, rotary and swarf kinds;
    /// `stamp_with` dispatches here exhaustively, so any other kind never
    /// reaches this match.
    pub(super) fn stamp_multiaxis(
        &self,
        world: &World,
        setup: Option<&str>,
        src: Option<&SrcRef>,
        h: &mut std::hash::DefaultHasher,
    ) {
        let wh = |h: &mut std::hash::DefaultHasher, v: Height| digest::wh(world, setup, src, h, v);
        let wh_opt = |h: &mut std::hash::DefaultHasher, v: Option<Height>| digest::wh_opt(world, setup, src, h, v);
        match self {
            // The 5-axis twins and plain swarf (tags 8/9/10): the
            // finish's scalar law with the pattern byte dropped - the
            // tag already names the kinematic - and the axis trio
            // appended, because leaning the spindle rewrites every
            // pose on the path. The raster kinds' optional stepover
            // rides the same Some/None split as the finish's: a
            // derived spacing must never digest as the number 0.
            OpParams::Contour5D {
                tolerance,
                clearance_z,
                retract_z,
                top_z,
                bottom_z,
                feed,
                plunge_feed,
                climb,
                lead_deg,
                lag_deg,
                max_tilt_deg,
                ..
            }
            | OpParams::Parallel5D {
                tolerance,
                clearance_z,
                retract_z,
                top_z,
                bottom_z,
                feed,
                plunge_feed,
                climb,
                lead_deg,
                lag_deg,
                max_tilt_deg,
                ..
            }
            | OpParams::Swarf {
                tolerance,
                clearance_z,
                retract_z,
                top_z,
                bottom_z,
                feed,
                plunge_feed,
                climb,
                lead_deg,
                lag_deg,
                max_tilt_deg,
                ..
            }
            // The rotary twins (M5.1 stage E, tags 12/13): the same
            // door's bytes - the axis the fit reads is model data,
            // not a parameter, so it enters through the facets' own
            // stamp. Only the stepover split keeps them apart.
            | OpParams::RotaryContour {
                tolerance,
                clearance_z,
                retract_z,
                top_z,
                bottom_z,
                feed,
                plunge_feed,
                climb,
                lead_deg,
                lag_deg,
                max_tilt_deg,
                ..
            }
            | OpParams::RotaryParallel {
                tolerance,
                clearance_z,
                retract_z,
                top_z,
                bottom_z,
                feed,
                plunge_feed,
                climb,
                lead_deg,
                lag_deg,
                max_tilt_deg,
                ..
            }
            // Flow (M5.1 I, tag 16) is the parallel's numbers whole,
            // the field read off the model and not a parameter: the
            // byte shape is the raster twins', one tag apart.
            | OpParams::Flow5D {
                tolerance,
                clearance_z,
                retract_z,
                top_z,
                bottom_z,
                feed,
                plunge_feed,
                climb,
                lead_deg,
                lag_deg,
                max_tilt_deg,
                ..
            }
            // The drive-curve contour (M5.1 J, tag 17) is flow's byte
            // shape with the stepover absent - it binds no stepover
            // field, so the read-back below simply skips it; the tag
            // alone separates the two digests. What is NOT a parameter
            // here is the path itself: the picked chains enter through
            // op.src's own digest, the selection's law.
            | OpParams::MultiAxisContour {
                tolerance,
                clearance_z,
                retract_z,
                top_z,
                bottom_z,
                feed,
                plunge_feed,
                climb,
                lead_deg,
                lag_deg,
                max_tilt_deg,
                ..
            } => {
                // The or-pattern cannot bind the optional stepover
                // (E0642: the contour has none) - read it back below.
                w(h, *tolerance);
                if let OpParams::Parallel5D { stepover, .. }
                | OpParams::Swarf { stepover, .. }
                | OpParams::RotaryParallel { stepover, .. }
                | OpParams::Flow5D { stepover, .. } = self
                {
                    match stepover {
                        Some(s) => {
                            h.write_u8(1);
                            w(h, *s);
                        }
                        None => h.write_u8(0),
                    }
                }
                wh(h, *clearance_z);
                wh(h, *top_z);
                wh(h, *bottom_z);
                for v in [*feed, *plunge_feed] {
                    w(h, v);
                }
                h.write_u8(u8::from(*climb));
                wh_opt(h, *retract_z);
                for v in [*lead_deg, *lag_deg, *max_tilt_deg] {
                    w(h, v);
                }
                // #2774: the radial heights of the rotary twins ride the
                // same read-back law, absent on every Z-plane op.
                if let OpParams::RotaryContour { clearance_r, retract_r, top_r, bottom_r, .. }
                | OpParams::RotaryParallel { clearance_r, retract_r, top_r, bottom_r, .. } = self
                {
                    for v in [*clearance_r, *retract_r, *top_r, *bottom_r] {
                        digest::wr_opt(h, v);
                    }
                }
                // The or-pattern cannot bind stage F's flag (the
                // swarf and rotary twins answer shank contact with
                // surface truth, not a free axis - it is not their
                // field to omit) - read it back the stepover's way.
                if let OpParams::Contour5D { shank_avoid, .. }
                | OpParams::Parallel5D { shank_avoid, .. }
                | OpParams::Flow5D { shank_avoid, .. }
                | OpParams::MultiAxisContour { shank_avoid, .. } = self
                {
                    h.write_u8(u8::from(*shank_avoid));
                }
                // #2772: the op's own stated rotation axis rides the
                // digest - written only when one is stated, so no
                // stamp of an op that follows the machine or the fit
                // moves (the machine's line enters through
                // `stamp.rs`'s own clause).
                if let OpParams::RotaryContour { stated_axis: Some(v), .. }
                | OpParams::RotaryParallel { stated_axis: Some(v), .. } = self
                {
                    h.write(b"rotary-op-stated-axis");
                    for c in v {
                        w(h, *c);
                    }
                }
            }
            // The flow band's digest (#2826, tag 48): the or-pattern
            // above cannot take it - the picks replaced the station
            // window - so its bytes stand alone, one tag apart. The
            // stated axis rides only when one is stated (the twins'
            // own law); the picked chains enter through op.src's own
            // digest, the selection's law.
            OpParams::RotaryFlow {
                tolerance,
                stepover,
                clearance_z,
                retract_z,
                feed,
                plunge_feed,
                climb,
                lead_deg,
                lag_deg,
                max_tilt_deg,
                stated_axis,
            } => {
                w(h, *tolerance);
                match stepover {
                    Some(s) => {
                        h.write_u8(1);
                        w(h, *s);
                    }
                    None => h.write_u8(0),
                }
                wh(h, *clearance_z);
                for v in [*feed, *plunge_feed] {
                    w(h, v);
                }
                h.write_u8(u8::from(*climb));
                wh_opt(h, *retract_z);
                for v in [*lead_deg, *lag_deg, *max_tilt_deg] {
                    w(h, v);
                }
                if let Some(v) = stated_axis {
                    h.write(b"rotary-op-stated-axis");
                    for c in v {
                        w(h, *c);
                    }
                }
            }
            // The wrap ladder, the stamped ladder and the advanced
            // swarf: the pocket door's arms, split by the 400-line law
            // (`multiaxis_pocket`).
            OpParams::RotaryOffset { .. } | OpParams::MultiAxisRough { .. } | OpParams::AdvancedSwarf { .. } => {
                super::multiaxis_pocket::pocket_arms(self, world, setup, src, h);
            }
            _ => {}
        }
    }
}
