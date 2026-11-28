//! The pocket door's multi-axis arms of `OpParams::stamp_with` (#1849
//! split; split from `multiaxis.rs` by the 400-line law): the wrap
//! ladder, the stamped ladder and the advanced swarf. Provenance:
//! moved verbatim from `multiaxis.rs`; our own digest law.

use super::*;
use super::digest::{self, w};

/// The digest bytes of the pocket door's multi-axis kinds: the bytes
/// `stamp_multiaxis` dispatches here, never reached by another kind.
pub(super) fn pocket_arms(
    params: &OpParams,
    world: &World,
    setup: Option<&str>,
    src: Option<&SrcRef>,
    h: &mut std::hash::DefaultHasher,
) {
    let wh = |h: &mut std::hash::DefaultHasher, v: Height| digest::wh(world, setup, src, h, v);
    let wh_opt = |h: &mut std::hash::DefaultHasher, v: Option<Height>| digest::wh_opt(world, setup, src, h, v);
    match params {
        // The wrap ladder (M5.1 stage E2, tag 14): the Pocket3D
        // door's bytes without the fields the kernel overrides or
        // never runs (retract, rest, feed-opt), the tolerance and
        // the three pose angles appended - the ladder's numbers,
        // read the wrap's way, digest as the ladder it is.
        OpParams::RotaryOffset {
            links: _,
            tolerance,
            stepover,
            stock_to_leave,
            clearance_z,
            top_z,
            bottom_z,
            clearance_r,
            top_r,
            stepdown,
            feed,
            plunge_feed,
            climb,
            strategy,
            entry,
            helix_radius,
            optimal_load,
            lead_deg,
            lag_deg,
            max_tilt_deg,
            stated_axis,
            picks: _,
        } => {
            // #2774: a radial height is a source word plus an offset;
            // the digest carries both so a change to either
            // regenerates once.
            for v in [*clearance_r, *top_r] {
                digest::wr_opt(h, v);
            }
            w(h, *stepover);
            w(h, *stock_to_leave);
            wh(h, *clearance_z);
            wh(h, *top_z);
            wh(h, *bottom_z);
            for v in [*stepdown, *feed, *plunge_feed, *helix_radius, *optimal_load] {
                w(h, v);
            }
            h.write_u8(u8::from(*climb));
            h.write_u8(match strategy {
                PocketStrategy::Zigzag => 0,
                PocketStrategy::Waterline => 1,
                PocketStrategy::Auto => 2,
                PocketStrategy::Adaptive | PocketStrategy::OffsetRings => 3,
            });
            h.write_u8(match entry {
                Entry3D::Plunge => 0,
                Entry3D::Ramp => 1,
                Entry3D::Helix => 2,
                Entry3D::Arc => 3,
            });
            // #2772: the op's own stated rotation axis rides the
            // digest - written only when one is stated, so no
            // stamp of an op that follows the machine or the fit
            // moves.
            if let Some(v) = stated_axis {
                h.write(b"rotary-op-stated-axis");
                for c in v {
                    w(h, *c);
                }
            }
            for v in [*tolerance, *lead_deg, *lag_deg, *max_tilt_deg] {
                w(h, v);
            }
        }
        // The stamped ladder (M5.1 stage E3, tag 15): the Pocket3D
        // arm's bytes verbatim - the hop plane included, the rest
        // and feed-opt switches absent (the kernel runs neither,
        // variant docs) - then the tolerance and the three pose
        // angles: leaning the spindle rewrites every pose on the
        // path, so it moves the stamp even though no position
        // moves.
        OpParams::MultiAxisRough {
            links: _,
            tolerance,
            stepover,
            stock_to_leave,
            clearance_z,
            retract_z,
            top_z,
            bottom_z,
            stepdown,
            feed,
            plunge_feed,
            climb,
            strategy,
            entry,
            helix_radius,
            optimal_load,
            lead_deg,
            lag_deg,
            max_tilt_deg,
            shank_avoid,
            picks: _,
        } => {
            w(h, *stepover);
            w(h, *stock_to_leave);
            wh(h, *clearance_z);
            wh(h, *top_z);
            wh(h, *bottom_z);
            for v in [*stepdown, *feed, *plunge_feed, *helix_radius, *optimal_load] {
                w(h, v);
            }
            h.write_u8(u8::from(*climb));
            h.write_u8(match strategy {
                PocketStrategy::Zigzag => 0,
                PocketStrategy::Waterline => 1,
                PocketStrategy::Auto => 2,
                PocketStrategy::Adaptive | PocketStrategy::OffsetRings => 3,
            });
            h.write_u8(match entry {
                Entry3D::Plunge => 0,
                Entry3D::Ramp => 1,
                Entry3D::Helix => 2,
                Entry3D::Arc => 3,
            });
            wh_opt(h, *retract_z);
            for v in [*tolerance, *lead_deg, *lag_deg, *max_tilt_deg] {
                w(h, v);
            }
            // The dodge rides the digest: the flag changes the
            // poses the stamp states, and a stale path must not
            // survive its flip (stage F).
            h.write_u8(u8::from(*shank_avoid));
        }
        // The ladder is the swarf door's bytes plus the rung count
        // (an integer on the digest rides `write_u32` like the tab
        // law's count above): two and three passes are different
        // programs, and the stamp says so.
        OpParams::AdvancedSwarf {
            tolerance,
            stepover,
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
            passes,
            picks: _,
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
            h.write_u32(*passes);
        }
        _ => {}
    }
}
