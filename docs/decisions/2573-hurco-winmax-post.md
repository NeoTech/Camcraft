## #2573 - Hurco WinMax (NC) ISNC post and Hurco VM10i machine

Sources: public Hurco WinMax Mill NC Programming Manual (`704-0116-101`),
`support.hurco.com` ISNC G-code/M-code reference pages, and the public Hurco
VM10i VMC specification sheet. Behaviour only; Conversational programs and the
older BNC dialect (`G70`/`G71`) are out of scope. No vendor post or code was
consulted.

- **Decided:** Emit `G21`/`G20` on its own line (`programUnits = "mm inch"`,
  `endUnits = "G20 G21"`), emit `G90 G17 G94 <wcs> G40 G49 G80`, and omit
  `G91.1`.
  **Why:** WinMax ISNC uses `G20`/`G21` (BNC uses `G70`/`G71`) and has no
  `G91.1`/`G90.1` word because `I/J/K` arc centers are unconditionally
  incremental from the arc start point.
  **Dropped:** Emitting `G91.1` (undocumented on WinMax ISNC and rejected by
  strict controls).
- **Decided:** Format dwell `P` as decimal seconds (`P0.5`, `P1.`) on `G04`,
  `G82`, and `G89`; refuse non-zero dwell by name on `G73`, `G83`, `G84`,
  `G84.2`, `G84.3`, `G85`, and `G86`.
  **Why:** In Hurco WinMax ISNC, a decimal `P` always means seconds, regardless
  of the machine's *Least Dwell Units* parameter (`0.001` vs `0.0001`). Integer
  milliseconds would change duration depending on control settings. Cycles
  without a documented dwell parameter refuse rather than dropping `P`.
  **Dropped:** Integer millisecond `P` words.
- **Decided:** Emit `G84.2` (right-hand) and `G84.3` (left-hand) for rigid
  tapping (`CycleType::Tap`), refusing `machine.rigidTapping === false` by name;
  emit `G84` for `CycleType::FloatingTap`.
  **Why:** WinMax ISNC defines `G84.2`/`G84.3` directly for synchronized rigid
  tapping and plain `G84` for floating tap holders, preventing silent rigid
  tapping on machines without it.
  **Dropped:** Pairing `M29` with `G84` for rigid tapping.
- **Decided:** Support `G54`..`G59` (1..6) and 93 auxiliary offsets
  `G54.1 P1`..`P93` (`workOffsetMax = 99`), refusing `> 99`; limit probing
  `G10 L2` updates to `P1..P6`.
  **Why:** Matches WinMax ISNC's 99 documented work coordinate offsets and
  `G10 L2` `G54`..`G59` range.
  **Dropped:** 254-offset Fanuc/LinuxCNC expansion beyond documented WinMax ISNC
  limits.
- **Decided:** Check `#3020` after every `G31` move. Touch moves require
  `#3020 = 1`. Guard moves require `#3020 = 0`.
  **Why:** Hurco documents this variable as the contact status of the move.
  The check applies in both inch and millimeter output.
  **Dropped:** Comparing the skip position with the expected contact position
  to determine whether contact occurred.
- **Decided:** Read the active source offset from `#2501..#2599` for X,
  `#2601..#2699` for Y, and `#2701..#2799` for Z before the probe update.
  **Why:** The [Hurco NCPP variable summary](https://support.hurco.com/wp-content/uploads/hurco_mill/NC_Programming/NCPP_Variable_Summary.htm)
  documents these registers and `#3020`. Each axis has six standard offsets
  followed by 93 auxiliary offsets. The source offset can differ from the
  `G10 L2` target.
  **Dropped:** Fanuc offset registers and a register stride of 20.
- **Decided:** Use `.nc` extension, note smoothing asks once (`SMOOTHING NOT
  WRITTEN - G05.2/G05.3 REQUIRE NC OPTION`) without emitting `G05.2`/`G05.3`,
  and add `hurco-vm10i.mch` (660 x 406 x 508 mm symmetric program-frame travel,
  28,000 mm/min rapid/feed, 12,000 RPM 11 kW CAT40 spindle with
  `rigid_tapping: true`, 24 tools, `Flood M8` / `Mist M7`, `safe_retract: 0.0`,
  `coordinate_rotation: true`).
  **Why:** WinMax Program Manager recognizes `.nc` natively (ISNC Save-As
  defaults to `.fnc`). `G05.2`/`G05.3` require optional NC Productivity Package
  support on the control.
  **Dropped:** `.fnc` default file extension and silent `G05.2` output.

**Simulator verification pending (PR #2646, latest review finding 2):**
No simulator access, load results, or path comparisons are available in this
session or on issue #2573. Programs were not checked on the community
owner's WinMax `09.01.393.30` simulator or on a physical Hurco machine.
Issue #2573 remains incomplete until each open dialect question has a short
program checked in that simulator. The issue must record the load result and
the comparison with the CamCraft path. Local tests do not replace these checks.

Cost to change later: low (`assets/posts/hurco-winmax.cps`,
`assets/machines/hurco-vm10i.mch`, two registry entries in
`crates/app/src/posting_files.rs`, and the test goldens).
