## #835 - a tool body is judged by its band stack, not the cylinder enclosing it

**Decided (option 1, by the orchestrating session; implemented here).**
Every verdict law that reads the tool's stack above the shank now reads
the same band stack the fixture laws sweep and the simulation draws
(`tool_body::stack`): law (c) shank/holder reach, (c3) the mid-turn
sweep, and every law that reads `verify::Cutter` - the solid law (b2),
the containment kinds' solid law, the posed rapids and the rotary field
law. The bands ride on the cutter (`Cutter::body`, a fixed-capacity
`BodyBands`, so the cutter stays `Copy` and `VerifyContext` gains no
field). Law (c) convicts a move once per group (the shank, then the
holder or the body's bands), worst offender first, so a body's many
bands report as the one holder they stand for.

**Kept the enclosing cylinder, said by name.** The drill-reach and
cross-drill refusals (one holder disc/tube by construction) and the
touch probe's stylus reach (stem and holder folded into one band): a
refusal that names the holder of a tool with a body gains "the tool
body is judged here by the cylinder enclosing it, Ø..."
(`operation::body_judged::name_enclosure`), and a probe path with a body
carries the same words as a note. Also unchanged, not verdicts on the
body's shape: the magazine fit (the changer's pocket takes the widest
diameter), the 5-axis shank-avoidance dodge (a generator choice; the
cylinder only leans more), and the posed-contact law's shank excuse.

**Why.** #743 asks that what is drawn be what is checked. The cylinder
over-reported: a stepped body (narrow nose under a wide flange) was
warned for a flange that stood above the wall. The band stack is never
smaller than the body (`HolderProfile`'s clipped envelope, the spun
profile's tangent-polygon arcs), pinned here by sampling an imported
mesh and a drawn profile with a cone, a convex and a concave arc.

**Guards.** A stack that is the plain holder cylinder is not carried
(no body, no change: byte-identical cutter and verdicts). A stack that
does not fit the tool's judged record (non-finite, in the flutes, wider
than the enclosing holder - or than the shank carried up to a raised
foot, steel the cylinder left out - or topping out elsewhere) is
refused and the cylinder stands in; `Cutter::judged` re-checks it.

**Contiguity is a checked law (review finding 2).** `BodyBands::of`
refuses a stack whose bands are not contiguous from the shank's top - a
gap, a late first band, or an overlap - and the enclosing cylinder
stands. Today's producers never make one; a gap band would otherwise go
unjudged.

**The raised foot (review finding 3).** When a body's foot stands above
the shank's top, the stack judges that gap at the shank's radius (the
shank carried up into the holder, `HolderProfile::stack`, as the fixture
laws have since #368), where the cylinder judged it at the enclosing
holder's radius. Direction: **looser** where the body is wider than the
shank, **stricter** where the shank is wider. Both pinned
(`tool_body_bands::a_raised_foot_over_a_slim_shank_is_judged_looser_than_the_cylinder`,
`..._over_a_fat_shank_is_judged_stricter_than_the_cylinder`). Kept on the
shank radius, because nothing modelled can be wider there: the body's
envelope starts at its own lowest vertex (`HolderProfile::envelope`'s
`z0`; a drawn profile at its first point), so no body steel stands below
its foot, and with a body the record's own holder is replaced by the
body (`tool_body::judged`) - it is not steel either. The only modelled
steel in the gap is the shank. Taking the body's widest radius there
would judge steel that is not in any model and would split the drawn
shape (the simulation draws the stack) from the judged one again.

**(c3) on a move (c) convicted (review note).** Law (c) used to stand the
mid-turn sweep down on every move it convicted. Both block the post, but
a stock-only `ShankHit` could hide a sweep through the PART on that move:
a different severity, and the user is told the wrong thing. Now the
sweep is skipped only where (c) already named the part; on a stock-only
conviction it still runs and its part finding is reported (a stock-only
sweep there is the same event and is dropped). This holds for tools
without a body too (it can only add a part warning). Pinned by
`tool_body_bands::a_swing_through_the_part_is_not_hidden_behind_a_stock_shank_hit`.

**Dropped.** Option 2 (keep the cylinder, explain it in the warning text
and draw it on demand): it leaves the drawn and judged shapes different,
which #743 asks to end. Moving drill reach and the probe stylus to bands
too: each needs its own per-band reading (a disc at each band's face;
the stylus's stem rebuilt under the body) - filed as follow-up work, not
a blocker, since the cylinder only over-reports.

**Cost to change later.** Small: `Cutter::with_body` is the one place
the bands are attached (`operation::generate`); dropping it restores
the cylinder everywhere.
