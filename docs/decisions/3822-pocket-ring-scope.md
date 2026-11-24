## #3822 - The ring links of #3747: which strategies, and the lift plane

**Decided** (the coordinator, with the maintainer away).

- **The 2D Pocket with Outside containment stays under the ring law.** Its
  area is round a part (`around_part`), and its Waterline and Offset rings
  are one ring set, as the 2D Offset's are. The law is safe there: each
  in-plane link is proved (the certified area, no crossing, air or cut
  floor), or it lifts with a named reason. Users get the fewer lifts at
  once. Pins (`pass_start_levels::pocket_outside`), on Waterline and Offset
  rings, Inside and Outside: no overcut (shape mask), no crossing feeds,
  each lift named, and no lift and no re-entry round a disc (a part that
  splits no ring).
- **Only the ring strategies link by the ring law.** One predicate,
  `ring_links::rings` (`Waterline | OffsetRings`), gates
  `ring_links::for_side` at both callers (`pocket.rs`, the plain source of
  `pocket_avoid.rs`). It reads the strategy of the passes that are posted.
  - Zigzag rows are no ring set. They keep the links that they posted
    before #3818. Pin: `zigzag_rows_keep_their_links_round_a_part` compares
    the digest of the moves round the dog bone, Inside and Outside, with
    the digest read on main before #3818 (`b2ff0ef1b`). The pin fails when
    the predicate is removed.
  - The Adaptive spiral is no ring set, so the ring law never links it.
    Where the HSM plan runs its own spiral round a part (Outside with a
    stock outline: the air plan of #3355), it posts the moves of main
    before #3818. Pin: `adaptive_spiral_round_a_part_outside_equals_main_before_3818`
    (the square fixture of `pocket_hsm_start_tests` with `around_part`,
    the digest read on `b2ff0ef1b`); it fails when the predicate is
    removed.
  - Where the HSM plan cannot start on the start side (a hole wall Inside;
    on the dog bone also Outside), it falls back to Offset rings by name.
    Those passes are rings, and the planner joins them by the in-air join
    law of #3747 (`level_order`). So they also link by the ring law, and
    the pins of the 2D Pocket apply to them:
    `adaptive_fallback_rings_link_by_the_ring_law` (the fallback is named,
    no overcut, no crossing feed, each lift named). A pass link by the old
    law after a join by the new law would mix two laws in one ring set.
- **A failed ring link lifts to the plane of the user's pass link.** With
  `Link::Clearance` it lifts to the clearance height; with `Retract`,
  `KeepDown` or the default (`Links::PLANAR`, `Retract`) it lifts to the
  retract plane, as before. The reason is still tallied and named, and the
  `link:` note names the plane ("lifted to the clearance height"). Pin:
  `failed_ring_link_lifts_to_the_users_pass_plane` (a clamp across the
  link, both ring strategies, all three links).

**Why.** A user who chose Clearance expects the clearance height for each
lift between passes; the ring law changes when the tool lifts, not where to.
Zigzag rows have no inside or outside; the ring law was written and
pinned for rings only (#3747, #3743).

**Dropped.**

- Narrow the gate to the 2D Offset op, so that the 2D Pocket waits for
  #3752: the Pocket's rings would keep their unnamed lifts for no safety
  gain.
- Read the predicate on the strategy before the HSM fallback, so that the
  fallback rings keep the old pass links: the planner joins would still
  use the new law (the Adaptive Inside digest still differs from main
  before #3818), and the gate of `level_order` reads no strategy.
- Keep `hop_z` for every failed link: it ignores the user's choice.

**Cost to change.** The predicate is one function; adding a strategy to it
needs that strategy's pins. The lift plane is one match arm in
`ladder_links::pass_link`.
