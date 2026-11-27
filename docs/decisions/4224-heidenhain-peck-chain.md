## #4224 - Heidenhain DIN/ISO: PECK and DEEP post the explicit chain

**Decided.** The Heidenhain DIN/ISO post (`assets/posts/heidenhain.cps`)
writes PECK and DEEP holes as the expanded chain in plain `G00` / `G01`
moves, at each R clearance. It writes no `G205` and no peck `G200`. A
single-plunge DRILL or SPOT keeps its native `G200` (its `Q202` is the
whole depth, so the cycle makes one infeed and one retract, as the chain
does). Taps keep `G206` / `G207`.

**Why.** The posted motion must be the motion that the simulation and the
gouge check read, and they read the chain.
- The chain for PECK (`crates/motion/src/drilling/cycle.rs`) feeds down
  by the peck, hops up by `CHIP_BREAK` (1 mm) at rapid, and feeds on from
  the hop. The old `G205` had `Q257` = `Q202` = the peck and `Q258=0`: a
  full retract after each infeed, then FMAX back to 0 mm above the last
  depth, which is the cut face.
- The chain for DEEP rapids to R and feeds down from R. Cycle 200 moves at
  FMAX to the set-up clearance above the last depth. The post wrote a
  native peck only with `Q200=0`, so that point was the cut face too.
- A native `G205` set that comes near the chain (`Q202` = the whole depth,
  `Q257` = the peck, `Q256` = 1) depends on how each control version does
  the chip-break retract and the re-approach. The cycle manuals that the
  post cites do not let us prove that motion equal to the chain. The
  target machine (iTNC 530 at the old cycle level) has no `Q208`, so its
  retract rate is not stated. The explicit chain is equal to the checked
  motion by construction, and the post already used it for every peck hole
  whose R is above the surface.

**Dropped.**
- A corrected native `G205` / `G200` (positive `Q258` / `Q200`, or `Q202`
  = the whole depth). Shorter programs, but the motion is not provably the
  chain, and DEEP's feed from R has no native equivalent.
- A refusal of PECK and DEEP on this post. It blocks legal work that the
  chain posts correctly.
- The `Q208` retract-feed machinery (`RETRACT_FEED_MAX`,
  `OLD_CYCLE_RETRACT_FEED`) existed only for `G205`, so it is removed. The
  level table stays: the `Q395` of `G200` still reads it, and its refusal
  of a `Q395` ask that the 340 49x cycle cannot take is kept and pinned.

**Cost to change later.** A native peck can come back when a test pins
its control motion against the chain (a control trace or a simulator of
cycle 205), and the chain generator can describe the cycle's own
re-approach. Then only the `PECK` / `DEEP` branch of `onCycle` changes.
