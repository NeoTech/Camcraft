//! #4451: the thread mill's tooth form in the tool library preview. The
//! steel outline of a thread mill is `Tool::profile`'s crest cylinder over
//! the cutting length (the envelope the reach laws read), so the preview
//! drew a large plain cylinder. Here the cutting length is drawn as the
//! teeth: the core at the tooth roots, and one V tooth of the form angle
//! for each row (a multi-form: rows x pitch; a single-form: one tooth of
//! the stated tooth height). The steel above the cutting length (the
//! shank, or the neck of a landed drawn cutter) is kept as it is.
//! Display only: no law, check, toolpath or post reads it, and every
//! point stays inside the crest cylinder, so the plain mesh's box fits.
//! Provenance: the ISO metric thread profile (ISO 68-1; ISO 965-1 / DIN
//! 13 external root depth 17/24 H = 0.6134 x pitch, as
//! `camcraft_tools::thread_mill`). No vendor material consulted.

use camcraft_app::thread_mill;
use camcraft_app::tools::{Tool, ToolKind};

/// The steel meridian `steel` (`(z, r)`, tip up) with the thread mill's
/// teeth drawn on its cutting length, or `None` when `t` is not a thread
/// mill or its numbers draw no tooth (the caller keeps the plain steel).
pub(crate) fn toothed(t: &Tool, steel: &[(f64, f64)]) -> Option<Vec<(f64, f64)>> {
    let teeth = teeth(t)?;
    let len = t.flute_len;
    // The steel from the top of the cutting length on: of the points at
    // that height only the last one, the radius the steel goes on with.
    let above: Vec<(f64, f64)> = steel.iter().copied().filter(|&(z, r)| z.is_finite() && r.is_finite() && z > len - 1e-9).collect();
    let first = above.iter().rposition(|&(z, _)| z <= len + 1e-9).unwrap_or(0);
    let mut out = teeth;
    out.extend(above.into_iter().skip(first));
    Some(out)
}

/// The tooth outline over `0..=flute_len`, tip first: `(0, 0)`, the core,
/// the teeth, ending on the core at the top of the cutting length.
pub(crate) fn teeth(t: &Tool) -> Option<Vec<(f64, f64)>> {
    if !thread_mill::is_thread_mill(t.kind) {
        return None;
    }
    let (r, len, pitch) = (t.diameter * 0.5, t.flute_len, t.pitch);
    let half = (t.tip_angle * 0.5).to_radians();
    let ok = [r, len, pitch, half].iter().all(|v| v.is_finite() && *v > 0.0) && t.tip_angle < 180.0;
    if !ok {
        return None;
    }
    // The tooth height: the external root depth at the (coarsest) pitch,
    // held under the crest radius so a core is always left.
    let depth = (thread_mill::EXTERNAL_DEPTH_PER_PITCH * pitch).min(r * 0.8);
    let core = r - depth;
    let (rows, step) = match t.kind {
        ToolKind::ThreadMill => (thread_mill::rows(t), pitch),
        _ => (1, len),
    };
    // The flank's axial run, so the V keeps the form angle; a tooth never
    // runs past its own row.
    let run = (depth * half.tan()).min(step * 0.5);
    let mut p = vec![(0.0, 0.0), (0.0, core)];
    for k in 0..rows {
        let mid = (f64::from(k) + 0.5) * step;
        p.extend([(mid - run, core), (mid, r), (mid + run, core)]);
    }
    p.push((len, core));
    p.dedup_by(|a, b| (a.0 - b.0).abs() < 1e-12 && (a.1 - b.1).abs() < 1e-12);
    Some(p)
}

#[cfg(test)]
mod tests {
    use super::*;
    use camcraft_app::thread_mill::thread_mill;

    fn radii_between(p: &[(f64, f64)], z0: f64, z1: f64) -> Vec<f64> {
        p.iter().filter(|q| q.0 >= z0 - 1e-9 && q.0 <= z1 + 1e-9).map(|q| q.1).collect()
    }

    #[test]
    fn a_multi_form_thread_mill_draws_one_tooth_per_row_on_its_core() {
        // M8 x 1.25, 10 mm cutting length: 8 rows.
        let t = thread_mill(6.0, 1.25, 0.0, 10.0);
        let p = teeth(&t).unwrap_or_default();
        let crests = p.iter().filter(|q| (q.1 - 3.0).abs() < 1e-9).count();
        assert_eq!(crests, thread_mill::rows(&t) as usize, "one crest per row");
        assert_eq!(crests, 8);
        let core = 3.0 - thread_mill::EXTERNAL_DEPTH_PER_PITCH * 1.25;
        let min = p.iter().skip(1).map(|q| q.1).fold(f64::INFINITY, f64::min);
        assert!((min - core).abs() < 1e-9, "the roots are the core: {min}");
        assert!(p.iter().all(|q| q.1 <= 3.0 + 1e-12), "no point leaves the crest cylinder");
        assert_eq!(p.last().copied(), Some((10.0, core)), "the teeth end on the core at the cutting length");
    }

    #[test]
    fn a_single_form_thread_mill_draws_one_tooth_then_its_neck() {
        // A single-form mill, 2 mm tooth, with a neck of 4 mm under a
        // 6 mm shank as a drawn cutter states it.
        let t = thread_mill(6.0, 1.5, 0.5, 2.0);
        let steel = [(0.0, 0.0), (0.0, 3.0), (2.0, 3.0), (2.0, 2.0), (14.0, 2.0), (14.0, 3.0), (42.0, 3.0)];
        let p = toothed(&t, &steel).unwrap_or_default();
        let crests = p.iter().filter(|q| (q.1 - 3.0).abs() < 1e-9 && q.0 < 2.0).count();
        assert_eq!(crests, 1, "one tooth: {p:?}");
        assert!(p.windows(2).any(|w| w[0] == (2.0, 2.0) && w[1] == (14.0, 2.0)), "the neck keeps its diameter and length: {p:?}");
        let teeth = radii_between(&p, 0.0, 2.0);
        assert!(teeth.iter().all(|r| *r <= 3.0 + 1e-12), "the tooth stays inside the crest: {teeth:?}");
        assert_eq!(p.last().copied(), Some((42.0, 3.0)), "the shank is kept");
        let at_len: Vec<f64> = p.iter().filter(|q| (q.0 - 2.0).abs() < 1e-9).map(|q| q.1).collect();
        assert_eq!(at_len.len(), 2, "the core steps once to the neck at the cutting length: {at_len:?}");
    }

    #[test]
    fn the_profile_shank_follows_the_teeth() {
        let t = thread_mill(8.0, 1.5, 0.0, 12.0);
        let steel = Tool { holder_len: 0.0, ..t.clone() }.profile();
        let p = toothed(&t, &steel).unwrap_or_default();
        let zs = t.flute_len + t.shank_len;
        assert!(p.iter().any(|q| (q.0 - zs).abs() < 1e-9 && (q.1 - 4.0).abs() < 1e-9), "the shank top is kept: {p:?}");
        assert!(p.windows(2).all(|w| w[1].0 >= w[0].0 - 1e-12), "the meridian rises: {p:?}");
    }

    #[test]
    fn other_kinds_and_bad_numbers_draw_no_teeth() {
        let mut t = thread_mill(6.0, 1.0, 0.0, 6.0);
        t.kind = ToolKind::EndMill;
        assert_eq!(teeth(&t), None, "an end mill");
        let mut t = thread_mill(6.0, 0.0, 0.0, 6.0);
        assert_eq!(teeth(&t), None, "no pitch");
        t.pitch = f64::NAN;
        assert_eq!(teeth(&t), None, "a NaN pitch");
        let t = thread_mill(6.0, 1.0, 0.0, 0.0);
        assert_eq!(teeth(&t), None, "no cutting length");
    }
}
