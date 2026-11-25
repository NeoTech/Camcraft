## #3748 - 2D Offset "Pass starts at": a selected Helix, Ramp or Plunge gives a warning; the automatic dead-end ramp is retired

**Decided.** The maintainer's clarification on #3748: "we are not
REMOVING the other lead in strategies, we should give warnings if they are
picked instead of the ones that should be used."

1. Helix, Ramp and Plunge stay lead-in types in the code, the project
   file, the UI and MCP. Arc (the #3746 entry from the air, with an arc in
   the cut plane) is the recommended type for Inside and Outside.
2. A start-side ring pass round a part whose user selected Helix, Ramp or
   Plunge uses that entry. Where the Arc entry from the air applies (the
   air room of `air_entry::AirRoom` exists), the ladder writes one note,
   and the operation posts one advisory `Warning::EntryNotRecommended`.
   The status line counts it. The text names the selected type, Arc and
   the reason in one sentence (`verify::entry_advice`): "Pass starts at
   Outside: the Helix entry is used as selected, but Arc is recommended,
   because an Arc entry starts in the air outside the stock where it fits
   and never plunges into the material". The warning does not hold the
   post: the path is verified like any other.
3. The Lead in / Lead out page of 2D Offset greys nothing out. With
   Inside or Outside and Helix, Ramp or Plunge selected, it shows the same
   sentence as a visible line under the Type selector.
4. The #3577 dead-end ramp (`level_ramp_entry::dead_end_ramp`, the
   `ramp_starts` marks, the `too_short_at` plan check and its
   `ramp_fallback_note`) is removed. It was an entry that the user did not
   select. On `main` it was live only for the pocket form of Outside
   (`!around_part`, no stock known); #3744 had already stopped it round a
   part, and #3790 the #3567 dead ends there. A later pass that no lead-in
   reaches now takes the pass link's entry, which is the user's
   (`#2265` restart). This supersedes the #3567 decision ("ramp entry at a
   dead end", gathered into `RECOMMENDED.md` at v0.26.0; issue branches do
   not edit that file): superseded by #3741 / #3748.
5. #3746's named stand-in for a selected Arc where no air entry fits (a
   helix, then the ramp along the first cut, each named; else the
   refusal) stays: it is named, not silent. No other silent change of a
   selected Arc into a helix, a ramp or an axial dive was found in the
   entry code (`descent::plan_core` names its stand-ins; the restart in
   `descent_dive` names its plunge).

**Why.** The maintainer's rule (#3741, #3748): an Outside cut does not
lead in by a helix or a ramp, but the user may still pick one, and the
operation must say that it is not the recommended entry.

**Dropped.**
- A new 2D Offset seed with Arc in place of Helix
  (`kinds.rs::seed_offset2d`). A new op starts at "Pass starts at" Auto,
  and the seed is the pocket-family default for every area, pockets
  included. The change reaches 17 test files of the app and UI crates
  that create a 2D Offset without an entry. Thus a user who sets Inside or
  Outside on a new op sees the advice at once and selects Arc. A follow-up
  issue, #3843, owns the seed.
- The advice for Auto: the user selected no start side. Auto keeps its
  path byte for byte.
- The advice for the 2D Adaptive twin and for a pocket area (no part, so
  no air room): there is no Arc entry from the air to recommend. The page
  shows the advice for 2D Offset Inside or Outside; it cannot tell a
  pocket area from a band round a part, so on a pocket area the page
  shows the line and the op posts no warning.
- Greying out Helix, Ramp and Plunge (the first form of #3748): the
  maintainer's clarification keeps them selectable.

**Cost to change later.** The text is one function
(`sim::verify::entry_advice`); the gate is the air room in `pocket.rs`
and `pocket_avoid.rs`. #3750 moves the entry rule into `ring_entry`.
