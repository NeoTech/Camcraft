## #3884 - Plain Outside pocket: Arc with no air room is refused; each stand-in entry and a selected entry is a warning

**Decided** (the coordinator's review decision on PR #3899, from the
maintainer's rule on #3741: "an Outside pass never uses a helix or ramp
entry; the tool goes down in air outside the stock").

1. **One Outside-air check for each Outside entry path.** A selected Arc
   on an Outside ring pass goes down in air outside the stock, or the op
   is refused by name (`OUTSIDE_AIR_REFUSAL`: "entry refused: no air room
   for an Outside entry: set a stock, or choose an Inside start or a
   Helix, Ramp or Plunge entry"). It never takes an automatic helix, ramp
   or dive. One function, `entry_warn::outside_air_check`, decides it, and
   each path calls it:
   - the plan (`pocket.rs`, `pocket_avoid.rs`): no air room at all (no
     stock known, not round a part, or the fixtures fill it). This
     includes an Adaptive op on its fallback to Offset rings (the second
     review found that it dived into stock at (49.5, -19.4));
   - the first entry of a level (`air_entry::stand_in`): no air lead fits;
   - the re-entry after a lift (`entry_warn::outside_arc_restart`, called
     by `descent_dive::entry_restart` and `descent_dive::restart`): no air
     lead reaches the pass start.
   The ring strategies (Waterline, Offset rings) are checked. Zigzag is
   not: its rows have no inside or outside, and the op says that the start
   side is not used. Its pass starts in material behave as on an Auto
   Zigzag pocket, with each stand-in named.
2. **A selected Helix, Ramp or Plunge runs, with its one warning (#3748).**
   The advice note (`EntryNotRecommended`) is written on each Inside or
   Outside ring pass, with or without an air room: the air room is not the
   gate.
3. **A selected Plunge into uncut stock.** A start-side pass link that
   plunges into a pass start in material writes `PLUNGE_INTO_STOCK_NOTE`.
   The op posts `Warning::PlungeIntoStock` only where it posts no
   `EntryNotRecommended` for the Plunge, so the Plunge is named once.
4. **Stand-in entries that remain** (Inside passes; the entries of a
   selected Helix, Ramp or Plunge that do not fit) write the marker
   `ENTRY_STAND_IN_NOTE`, and the op posts one advisory
   `Warning::EntryStandIn { at, entries }`. `entry_status_lines` shows each
   stand-in note as a `warning:` line.

**Why.** Prime Directive: an entry that the user did not select and that
cuts into stock is refused, or it is visible. A warning on a ramp that
still cuts into stock on an Outside pass breaks the #3741 rule. The
refusal names the three fixes a user can make.

**Dropped.**
- The first form of this PR: keep the ramp stand-in on the Outside pass
  and post a warning. The review held that it breaks the #3741 rule.
- The second form of this PR: refuse only at plan level. The safety
  review found three paths past that gate: the first entry (helix, then
  ramp stand-in), the re-entry after a lift (ramp, then helix) and the
  Adaptive ring fallback (axial dive).
- Refuse Arc on Inside with no air room: the #3741 rule names Outside
  only. Inside keeps its named stand-in with the warning.
- Make the warnings block the post: the entries are verified, and the
  selected entry is the user's.
- Warn on the stand-ins of Auto ladders: out of scope. Their notes are
  warning lines on the op page, but they post no operation warning.

**Cost to change later.** The rule is in `machining2d/entry_warn.rs`
(the refusal, the note markers, the stand-in tails and
`entry_warnings`). The tails match the note texts of `descent_dive`,
`descent` and `air_entry`; a new stand-in note adds its tail there.

### Maintainer decision (owner's standing word 2026-11-23: choose the recommended option while he sleeps)

- **Chosen rule.** An Outside Arc re-entry after a lift comes down in air
  outside the stock, or on floor that is already cut at that depth. It
  never ramps, never takes a helix and never plunges into uncut stock.
  Else the op is refused by name (`OUTSIDE_AIR_REFUSAL`).
  `entry_warn::outside_air_check` decides it for every Outside entry path;
  `entry_warn::outside_arc_restart` finds the cut floor.
- **"Already cut"** means cut earlier in the program order (the moves
  posted before the re-entry), at or below that depth, by the whole tool
  disc: the centre, a ring at half a radius and the rim of the disc round
  the plunge point each lie within one tool radius of an earlier cut's
  centre path. A cut that comes later in the order, or one that covers
  only part of the disc, does not count. The link at depth from that
  point to the pass start runs through cut floor up to one tool radius
  short of the start, and crosses no earlier cut (the Crossing law). Pins:
  `outside_reentry_onto_cut_floor_is_accepted`,
  `outside_reentry_onto_a_partly_cut_disc_is_refused`,
  `outside_reentry_onto_floor_cut_later_is_refused`,
  `outside_reentry_without_an_air_lead_is_refused`.
- **Reason.** Without the cut floor, fig8 2D Offset Outside with Arc is
  refused completely: four pass links (#3825) lift to pass starts inside
  ring 0 that no air lead reaches. That is the main Outside case of the
  maintainer.
- **Option dropped.** Refuse every Outside re-entry that is not in air
  outside the stock.
