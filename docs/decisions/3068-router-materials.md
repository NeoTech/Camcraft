## #3068 - Router materials: wood, MDF, plywood, acrylic and HDPE

**Decided.**

- Six work materials are added to the cutting table: `softwood`,
  `hardwood`, `mdf`, `plywood`, `acrylic` and `hdpe`. They are appended
  after the six metal and POM rows. The table is indexed in `ALL` order,
  and a project file stores the word, so old files read the same.
- The new rows get their chip load from a router curve by cutter
  diameter (`camcraft_tools::cutting_router::router_scale`). The points
  are 3 mm 0.30, 6 mm 0.65, 10 mm 1.0 and 12 mm 1.15 of the 10 mm chip
  load. Below 3 mm the scale falls in proportion to the diameter. Above
  12 mm it holds.
- On a router material, a V-bit (`ToolKind::Taper`) reads its chip load
  at half its diameter. A V-bit cuts mostly near its tip.
- The turning table gets the same six rows, because it is indexed in
  `ALL` order too.
- The rpm clamp is the existing one in `app::cutting::compute`. It
  already recomputes the feed at the clamped rpm, so the chip load holds,
  and it names each clamp in `notes`. No change was needed there.

**Why.** The metal bands of `fz_scale` give every cutter from 3 to 6 mm
the same chip load. For wood that is about two times too much for a
1/8 in cutter, which breaks. Public router chip-load charts show a chip
load that grows with the diameter. All numbers are our own, at the low
side of the ranges that public woodworking and plastics sources print.

**Dropped.**

- One continuous curve for all materials. It would change the posted F
  of the metal rows and move goldens without a request.
- A separate chip-load table for each router material. One shared curve
  times a 10 mm value for each row and tool material gives the same
  shape with fewer numbers to check.
- The half-diameter V-bit reading for metal rows. It would change the
  metal output; it can follow in its own issue.

**Cost to change.** The numbers are constants in two tables and one
curve. The pinning tests in `crates/app/tests/cutting_router.rs` state
the hand calculation, so a change of a number changes one test line.
