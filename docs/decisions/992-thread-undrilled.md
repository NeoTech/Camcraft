## #992 - Thread milling refuses a hole the program has not drilled yet

**What was decided**

1. **The prior grid judges the way in.** At generate, after the thread ride
   is built, each threaded hole is read in the stock the setup's earlier
   operations leave standing - the same lazily cut grid the drilling and
   rapid laws read (`operation::drill_prior::PriorCell`, from
   `job::collect_job`'s `verify_prior`). The highest stock standing over
   the read disk must be at or below the lowest point the path takes the
   mill to inside that hole; otherwise generate refuses by name: "the hole
   at (x, y) is not drilled yet: the stock the program ahead leaves stands
   at z .. inside it, above the thread mill's lowest point at z .. - a
   thread mill does not cut at its centre - add a drilling operation of at
   least the .. mm minor diameter, deep enough for its point, before the
   thread" (`operation::thread_entry::judge`).
2. **The disk read is the minor circle less one grid cell, never narrower
   than the mill.** The ride assumes the hole is open to the minor
   diameter: the descent runs on the axis, and the lead-in and every
   radial pass sweep the mill out to the thread. A drill well under the
   minor diameter would leave a wall the thread mill's teeth would have to
   cut at thread feed before the first thread pass. One cell of slack is
   the lattice's own resolution: the sweep cuts a column whose centre lies
   inside the drill's disk, so a drill of exactly the minor diameter is
   read open. For the tight case (an M8 x 1.25 with a 6 mm mill) the read
   falls back to the mill's own cylinder, which is what the issue's
   "a Drill of at least the tool's diameter generates" meant.
3. **Unknown is refused.** Nothing ahead cut (no grid at all), a program
   ahead the column grid cannot cut, a posed (3+2) Thread operation, a
   carried body whose prior is distrusted, a hole off the grid, or a
   non-finite reading: each is refused by name. An unjudged hole is never
   air.
4. **The lowest point is read from the path, not the kernel's arithmetic.**
   The lowest Z of any move end inside the hole's minor circle is the
   mill's lowest point there - one law for blind, through and stated
   bottoms, and for any future ride change.
5. **Staleness needed no new code.** An unposed operation's stamp already
   digests the program ahead (`drill_prior::reads_prior` is true for every
   unposed kind, #109), so moving the Drill after the Thread marks the
   Thread Stale and its regeneration refuses. Pinned, not re-built.
6. **Fixtures model the tap-drill hole deeper than the thread.** The #894
   thread suites drew the hole exactly at the thread's floor, so a drill
   ahead would leave its point under the thread mill. The shared fixture
   (`tests/thread_support`) now draws the hole to z 4 (the tap drill's
   depth) and threads to z 8 as before: the thread paths and their CL
   goldens are unchanged; the M8 NC golden gains the Drill operation ahead
   (and the thread mill becomes T02). The floor-gouge test drills past the
   modelled floor so the floor law, not this refusal, is what it pins.
   `mill2d_c`'s R10 thread gets a 20 mm drill ahead; the MCP thread test
   now pins generate's refusal of the undrilled block.

**Why**

A thread mill is not centre-cutting and the ride feeds it down the hole's
axis at plunge feed. With #894 a Thread operation posts, so a forgotten tap
drill was a broken cutter with no finding at generate, verify or the post
gate (the gouge laws read the part, where the hole is modelled). The
Prime Directive puts this at P0 and asks for a refusal by name.

**Sources (public)**

- Machinery's Handbook, "Thread milling": the hole is drilled to the
  tap-drill (minor) diameter first; thread mills cut on their side teeth.
- Peter Smid, "CNC Programming Handbook", thread milling chapter: drill
  depth exceeds the thread depth so the cutter clears the drill point.

No HSMWorks / Fusion material, output or documentation was consulted.

**Considered and dropped**

- *Asking the centre-cutting plunge law (`plunge_law::audit`)*: it reads
  the raw stock, so every thread would be refused; it has no prior-grid
  reading.
- *Reading only the mill's own cylinder (the issue's sketch)*: lets a
  drill much narrower than the minor diameter (6 mm for an M10) pass, and
  the mill then side-cuts a full wall at thread feed - a likely breakage.
- *The padded disk the drilling laws read (half a cell's diagonal past the
  radius)*: refuses a drill of exactly the mill's diameter, which the
  issue says must generate.
- *Falling back to the raw stock when nothing ahead cut* (a cast part with
  the hole cored): needs a stock grid on the job thread with no world;
  refused instead, rare, and the refusal names what to add.

**What it would cost to change later**

- A cored-stock acquittal: build the raw stock grid when no plan exists
  and read it the same way (one branch in `thread_entry::judge`).
- Posed thread milling: needs a frame-aware prior reading; today a posed
  Thread is refused.
- The milled Bore ride's axis entry has the same gap and is filed apart (#1036).
