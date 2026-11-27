//! Per-`State` cache of each body's world-space vertices (#2201). An entry
//! keys on its inputs: the facet allocation (a `Weak` compared by pointer,
//! so a replaced or released mesh never hits) and the pose bits. No
//! document counter is part of the key (crates/ui/AGENTS.md, #2238).
//!
//! Provenance: our own per-frame scene cache; no external code or vendor material.

use std::sync::{Arc, Mutex, Weak};

use camcraft_app::geometry::Triangle;

const WORLD_VERTS_CAP: usize = 32;

/// The bits of a body's view pose: position, then the rotation quaternion.
#[must_use]
pub(crate) fn pose_bits(loc: &camcraft_app::schema::Location) -> [u64; 7] {
    [
        loc.pos.x.to_bits(),
        loc.pos.y.to_bits(),
        loc.pos.z.to_bits(),
        loc.rot[0].to_bits(),
        loc.rot[1].to_bits(),
        loc.rot[2].to_bits(),
        loc.rot[3].to_bits(),
    ]
}

struct WorldVertsEntry {
    facets: Weak<[Triangle]>,
    facets_len: usize,
    pose: [u64; 7],
    verts: Arc<Vec<[f32; 3]>>,
}

#[derive(Default)]
struct FrameCacheInner {
    world_verts: Vec<WorldVertsEntry>,
    builds: u64,
}

/// Interior-mutable vertex cache held on `State`.
#[derive(Default)]
pub(crate) struct FrameCache {
    inner: Mutex<FrameCacheInner>,
    /// The path lines turned by the playhead's work turn (#4266).
    pub(crate) turned_lines: TurnedCache<[f32; 6]>,
    /// The model, stock and fill vertices turned by it.
    pub(crate) turned_verts: TurnedCache<[f32; 3]>,
    /// The toolpath overlay lines and the chevron arm (#2201).
    pub(crate) paths: crate::path_overlay::PathLineCache,
    /// The DXF profile overlay lines (#1863).
    pub(crate) sketches: crate::sketch_overlay::SketchLineCache,
    /// The ground shadow mesh and the occlusion values (#2834).
    pub(crate) shade: crate::shade_cache::ShadeCache,
}

fn lock_inner(cache: &FrameCache) -> std::sync::MutexGuard<'_, FrameCacheInner> {
    cache.inner.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

impl FrameCache {
    /// The cached vertices of `facets` at `pose`, or `build`'s result
    /// stored under that key. Dead entries are pruned on each call.
    pub(crate) fn world_verts_cached(
        &self,
        facets: &Arc<[Triangle]>,
        pose: [u64; 7],
        build: impl FnOnce() -> Vec<[f32; 3]>,
    ) -> Arc<Vec<[f32; 3]>> {
        let matches = |e: &WorldVertsEntry| {
            e.facets.strong_count() > 0
                && std::ptr::eq(e.facets.as_ptr(), Arc::as_ptr(facets))
                && e.facets_len == facets.len()
                && e.pose == pose
        };
        {
            let mut inner = lock_inner(self);
            inner.world_verts.retain(|e| e.facets.strong_count() > 0);
            if let Some(i) = inner.world_verts.iter().position(matches) {
                inner.world_verts[i..].rotate_left(1);
                if let Some(entry) = inner.world_verts.last() {
                    return Arc::clone(&entry.verts);
                }
            }
        }
        let verts = Arc::new(build());
        let mut inner = lock_inner(self);
        inner.builds = inner.builds.saturating_add(1);
        if let Some(i) = inner.world_verts.iter().position(matches) {
            return Arc::clone(&inner.world_verts[i].verts);
        }
        if inner.world_verts.len() >= WORLD_VERTS_CAP {
            inner.world_verts.remove(0);
        }
        inner.world_verts.push(WorldVertsEntry {
            facets: Arc::downgrade(facets),
            facets_len: facets.len(),
            pose,
            verts: Arc::clone(&verts),
        });
        verts
    }

    /// How many vertex builds this cache ran (the counter pin).
    #[cfg(test)]
    pub(crate) fn world_verts_builds(&self) -> u64 {
        lock_inner(self).builds
    }
}

#[cfg(test)]
#[path = "frame_cache_tests.rs"]
mod tests;

// ------------------------------------------------------------ work turn

/// The work turn's bit pattern as the turned caches key it
/// (`crate::work_turn::NO_TURN` when there is none).
pub(crate) type TurnBits = [u64; 8];

const TURNED_CAP: usize = 64;

/// One turned-geometry entry: keyed on the source's identity (its
/// allocation pointer and length, a probe of its first and last
/// elements) and the work turn's bits (crates/ui/AGENTS.md "Frame
/// cost": the key is the input, never a document counter). The probe
/// and the bounded table keep a recycled allocation from serving a
/// foreign answer: a hit must match them all.
struct TurnedEntry<T> {
    src: u64,
    len: usize,
    view: TurnBits,
    probe: [u64; 6],
    verts: Arc<Vec<T>>,
}

/// The cache of source geometry under the playhead's work turn (#4266):
/// the transformed vertices are computed when an input changes and read
/// on every other frame, a paused playhead included.
pub(crate) struct TurnedCache<T> {
    inner: Mutex<TurnedInner<T>>,
}

struct TurnedInner<T> {
    entries: Vec<TurnedEntry<T>>,
    builds: u64,
}

impl<T> Default for TurnedCache<T> {
    fn default() -> Self {
        TurnedCache { inner: Mutex::new(TurnedInner { entries: Vec::new(), builds: 0 }) }
    }
}

/// The identity probe of a vertex source: its first and last
/// elements' bit patterns.
pub(crate) fn probe3(a: [f32; 3], b: [f32; 3]) -> [u64; 6] {
    let mut out = [0u64; 6];
    for (i, f) in a.into_iter().chain(b).enumerate() {
        out[i] = u64::from(f.to_bits());
    }
    out
}

/// The same for overlay lines (`[x, y, z, r, g, b]`): the first and the
/// last line's floats, two per slot.
pub(crate) fn probe6(a: [f32; 6], b: [f32; 6]) -> [u64; 6] {
    let mut out = [0u64; 6];
    for i in 0..6 {
        out[i] = u64::from(a[i].to_bits()) | (u64::from(b[i].to_bits()) << 32);
    }
    out
}

impl<T: Send + Sync + 'static> TurnedCache<T> {
    /// The source geometry `src` (`src` names its allocation, `probe`
    /// its ends) turned by `view`, computed once and read after: `Arc`
    /// vertices whose key is the input itself. The turn's writer (the
    /// playhead move) invalidates by changing the bits; no page or
    /// widget holds this cache.
    pub(crate) fn turned(
        &self,
        src: u64,
        len: usize,
        view: TurnBits,
        probe: [u64; 6],
        build: impl FnOnce() -> Vec<T>,
    ) -> Arc<Vec<T>> {
        let matches =
            |e: &TurnedEntry<T>| e.src == src && e.len == len && e.view == view && e.probe == probe;
        let mut inner = self.inner.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(i) = inner.entries.iter().position(matches) {
            inner.entries[i..].rotate_left(1);
            let last = inner.entries.len() - 1;
            return Arc::clone(&inner.entries[last].verts);
        }
        let verts = Arc::new(build());
        inner.builds = inner.builds.saturating_add(1);
        if inner.entries.len() >= TURNED_CAP {
            inner.entries.remove(0);
        }
        inner.entries.push(TurnedEntry { src, len, view, probe, verts: Arc::clone(&verts) });
        verts
    }

    /// How many turn builds this cache ran (the counter pin).
    #[cfg(test)]
    pub(crate) fn builds(&self) -> u64 {
        self.inner.lock().unwrap_or_else(std::sync::PoisonError::into_inner).builds
    }
}
