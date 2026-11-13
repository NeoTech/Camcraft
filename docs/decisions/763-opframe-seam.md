# #763 - One machining-frame seam: OpFrame composes part_alignment and TiltedFrame

## What was decided

Phase 1 ships `OpFrame` (`crates/app/src/operation/frame.rs`) as the single
machining-frame seam, composing the two existing frame mechanisms — the
setup's `part_alignment` (#326) and the op's 3+2 `Tilt` (M5.1 B) — with no
third mechanism. `collect_job` reads material and heights through it and
builds its tilt frame from `OpFrame::checked_frame`, one construction
site. No refusal lifted, no post change, no generator change; behavior is
byte-identical (the zero-tilt index reproduces the prismatic path
exactly, move for move).

The contract, stated once in the module header: three coordinate systems
(D document, S setup = the CL/post world, L local machining plane);
point maps applied right-to-left — `p_L = T(A(p_D))` with `A` the
alignment baked into part reads and `T` the tilt's `to_local`; the CL
wraps back L → S via `TiltedFrame::world_path` (a clone with no tilt).
Setup furniture (stock, WCS, fixtures) is stored in S: the alignment
never applies to it, the tilt reads it local.

Two seams ship correct from day one even though only tests exercise
them yet: heights and bounding boxes in L resolve against FULLY
TRANSFORMED geometry (`aabb_local_of`, `height_z_local`) — never a
`z − origin.z` shift of the S box, which is only correct while the tool
axis is the world Z (the user's condition 2, pinned by
`aabb_local_is_the_tight_box_of_transformed_geometry`).

## Why

`generation_problem` allowed exactly four kinds in an aligned setup
because the two frame mechanisms were built independently and refuse to
combine — a verification gap, not a mathematical one: they compose as
plain function composition. Building the seam first (a) makes the later
phases (picks, more kinds, alignment × tilt) read/wrap changes at ONE
place instead of scattered call sites, (b) pins the composition math
numerically before any refusal lifts, and (c) keeps every existing
behavior byte-identical while it lands.

## Considered and dropped

- **Carrying the rotated work offset instead** (#760's original heavy
  part): for normal 3-axis work, alignment coverage supersedes
  offset-rotation carry — the part can be aligned, the offset stays a
  plain translation, the post is untouched. #760 stays parked and is
  re-evaluated after this seam lands (its phase 4).
- **An `align` field on `OpFrame`** (holding the resolved
  `part_alignment`): rejected as a second copy that can drift — the
  alignment stays the setup's own state, read by the `setup_align`
  helpers through the seam's S-frame methods. One owner.
- **Routing the boundary/region builders through the seam in Phase 1**:
  deferred to the later phases; Phase 1 wires `collect_job`'s reads
  (material, heights, frame) only, so the diff stays reviewable and
  behavior-identical.
- **A new end-to-end post-gate test for tilted output**: the gate ("a
  tilted (A/B/C rotary) move is not supported") is already pinned
  verbatim by `tests/inverted_post.rs`; Phase 1 changes no gate, so the
  existing pin plus the suite run carries it. Phase 2+ adds its own
  when a tilt path actually moves.

## What it would cost to change later

The frame contract (D/S/L, right-to-left composition, furniture law) is
now the stated interface for every later phase — picks in aligned
setups, more kinds, alignment × tilt. Changing the composition order or
the furniture law later means re-deriving every numeric pin in
`opframe_seam.rs` and the seam's unit tests, and re-reviewing which
refusals can still lift. Machine kinematics (parked by the user) would
compose ON TOP of this seam, not beside it.
