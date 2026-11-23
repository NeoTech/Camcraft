## #1883 - Waterjet Profile operation and the shared 2D sheet kernel

**What was decided.**

1. A new module `motion::sheet` holds the 2D sheet kernel. It contains:
   - `nest` (contour roles and the inner-first cut order) and `ring`
     (kerf ring and walks), moved from `plasma`;
   - `laws` (the kerf law and the cover law) and `lead` (lit pieces,
     straight and arc leads), moved from `plasma`;
   - `arclen` (a ring by arc length), moved from `wire_edm::tabs`;
   - `open` (the one-sided mitred offset of an open run), moved from
     `wire_edm::open`.

   Each refusal names its operation and tool through a `Voice`, so that
   the plasma and wire EDM messages stay word for word. The plasma golden
   CL and all plasma and wire EDM tests pass without a change.
2. The cover law now takes one span or more for each contour (one for
   each stretch between tabs), and an `exempt` closure for the edge
   points that a tab leaves uncut on purpose. Plasma passes "no point is
   exempt". The law now refuses a closed contour that has no span.
3. `motion::waterjet::waterjet_profile` cuts as follows:
   - inner contours first;
   - the kerf ring at kerf / 2 on the waste side;
   - straight or arc leads with the pierce in the scrap. A lead that has
     no room is halved up to four times;
   - the tight-hole rule. A hole whose lead has no room after the
     halving is pierced at its centre and led in straight.
   - tabs as uncut gaps. A tab is laid on the nearest closed contour.
     Each stretch between gaps gets its own pierce and its own leads.
   - open runs on the centre line, or on the left or right side.
   - optional G41/G42, with the plasma's straight-entry rule;
   - optional arc linearization through `Move::chords`.

   The kerf and cover laws certify the result.
4. `OpType::WaterjetProfile` uses the stored id 46 and
   `Section::Waterjet` (`Family::Waterjet`). It mounts a `ToolKind::Waterjet`
   nozzle only. `.camproj` stores it as `waterjet_profile` (additive).
   MCP reaches it through the generic `add_operation` / `edit_operation`.
5. The op stores its cut data as explicit numbers: cut feed, pierce time,
   abrasive delay, and an optional kerf override (the nozzle's diameter
   by default). The pierce time and the abrasive delay go into the path's
   notes only. The CL record that carries them to the post is #1882.
6. A tabbed contour, a centre-pierced hole and an open run use the
   computer's offset under control compensation, and a note says so.
   Each stretch between tabs is too short to be a safe G41/G42 span.

**Why.** The maintainer asked for an end-to-end WAZER path (comment on
#1883, 2026-11-20). The issue asks for the kernel to be shared "without
duplicating geometry logic". Moving the code without a change, behind a
voice parameter, keeps the plasma and wire output byte for byte (the
goldens prove it). The waterjet then reuses the same certified laws.

**Considered and dropped.**

- Reuse of `wire_edm::tabs::tabbed` for the waterjet tabs. The wire
  threads once and retraces its kerf to each stretch, and it adds a tab
  cut-off pass. A waterjet pierces each stretch. Only the arc-length ring
  is common, and that ring is now shared (`sheet::arclen`).
- A waterjet CL record (`WaterjetCut`, jet spans). That record is #1882's
  scope. Until #1882 merges, the path carries notes only, and no post
  serves `Kind::Waterjet`.
- Selection of the cut data from the nozzle's recipe (`waterjet_row_for`
  by stock material and sheet thickness). The generation job does not
  carry the stock material, and `operation/job.rs` is changed by many
  open PRs. This is a follow-up issue.
- Refusal of control compensation together with tabs. Riding the
  computer's offset with a note is safer and does not refuse a legal
  program.

**Cost to change later.** The stored id 46 and the `waterjet_profile`
file key are permanent. An open PR that also appends a kind takes the
next free id when it merges second. The cut data can move to the recipe
without a file change: an absent field can mean "read the recipe".
