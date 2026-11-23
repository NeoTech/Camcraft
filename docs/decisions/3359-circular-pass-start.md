## #3359 - Circular pass order and vault entry

**What was decided.**

- Circular uses the shared `PassStart` type and the `pass_start`
  project key. Auto is omitted and preserves the existing operation
  stamp. The Passes tab and MCP edit use the same undoable command.
- Outside starts with the largest cutting ring. Inside starts with the
  smallest positive ring. Each ring completes its depth ladder before
  the next ring enters. The tool lifts vertically to clearance before
  it traverses to a different ring.
- In forced orders each ring starts its depth ladder from the material
  cap. Stock above Top Z receives depth steps to Top Z. The ring then
  steps to its floor. On a vault the floor is the paraboloid height at
  the outer reach of that ring's cutter disk. At Top Z the rim ring
  cuts only when material stands above it.
- A forced vault limits cutter centres for entries to the active ring's
  disk. A straight lead lies inside that convex disk when its endpoints
  do. For the helical lead, the distance of its posted centre from the
  vault axis plus the larger centre-to-endpoint radius bounds the
  whole sweep. Check the original near point and the two lattice
  points. An arc must also meet the post's minimum chord resolution.
  Where the requested entry cannot be flown, retain the existing
  named fallback and refusal laws. Helix crossing checks read the
  descents in the order flown.
- Keep the conservative material cap unless the app has local
  clearance evidence. A remaining height from carried stock can
  bound a cut only over the region it covers. Lowering a raw global
  cap for the traverse would need prior-cut proof over the cutter's
  swept lateral route, including the entry disc. The existing app
  verifier judges that route against stock, the part and the assembly.
- The ring order selects a radial side. Entry phase and eventual
  start and exit points (#1448) are independent. The first ring can
  cut the full tool width in solid stock. The operation reports that
  condition as a possibility.

**Why.** The first inner vault ring in an unreversed legacy ladder
would open at the preceding outer ring's floor. That premise is absent
when the inner ring runs first. The same risk applies to any uncut strip
outside previously cut material. A bounded ladder from the material cap
and an entry contained in the active band make each ring independent of
which neighbor ran first.

**Considered and dropped.**

- Reverse the legacy ladder. It can skip layers of solid stock.
- Certify a vault entry against the full cutter-centre disk. A helix can
  fit that disk and reach below the bowl on its outer sweep.
- Traverse between rings at depth, or use a diagonal lift. The next
  ring can still be in solid stock.
- Use the ring order to change its start point. A start point belongs
  to the entry geometry of a single ring.

**Hand trace.** The part uses a 10 mm target radius with a 6 mm cutter
and 2 mm stepover. The flat floor cuts radii 7, 5, 3 and 1 mm in Auto
and Outside, or 1, 3, 5 and 7 mm in Inside. Every ring cuts from
Z18 to Z0 in 2 mm steps. Every inter-ring traverse starts after a
vertical lift to Z30. With a vault from Z20 to Z0, the R7 floor is
Z20. At a stock top of Z20 that ring cuts air. The R5, R3 and R1
floors are Z12.8, Z7.2 and Z3.2. A forced Inside R1 ring starts with
Z18 and ends at Z3.2. A test reads the full emitted ring sequence,
approaches and clearance transfers.

**Cost to change.** A link below clearance or a ladder starting below
the material cap would need a proof of the preceding cuts for that ring
and approach. A different ring finish would need a new bound for the
whole cutter footprint. Keep the default CL goldens and verify both
forced orders, including the remaining material and the post gate.
