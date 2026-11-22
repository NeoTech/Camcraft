## #3300 - A leaning 5-axis path states its rotaries before the first tilted feed

**Decided.** The fix has two parts, one in each crate that owns a half of
the contract:

1. The 5-axis stamp (`crates/motion/src/multiaxis/stamp.rs`): when a path
   leans anywhere, each station before its first lean states the upright
   Z column (`Some(UP)`) instead of no pose. A path that never leans stays
   silent, as before.
2. The post dump (`crates/post/src/post/mod.rs`, `dump`): in a stream that
   leans, the first move that states the Z column commands explicit zero
   angles (A0 B0 C0, filtered to the letters the machine has). Before, it
   commanded the zero only after a lean.

The result: the opening rapid of the program (at the clearance, after the
post's own retract to the clearance) carries A0 B0. The Fanuc angle memory
(`rotAt`) is then known, and the first tilted feed is capped at the stated
rotary rate instead of refused.

**Why.** The guard in `rotaryFeed` must stay: a feed that turns a rotary
from an angle the post does not know is refused. So the fix must make the
angle known, not relax the check. The generator is the only party that
knows the path starts upright on purpose, so it states that pose. The post
commands only a pose that the stream states: a stream that states no pose
before its first tilted feed (a hand-written feed-first stream) still
refuses. The stamp alone was not enough (the released claim on the issue
found this): the dump wrote nothing for a stated Z column before a lean.

**Dropped.**

- Post dump only (zero angles on the first move of any stream that leans,
  stated or not): the post would guess an upright home for a move that
  states nothing. That is the guess the guard forbids.
- Fanuc script only (start `rotAt` at zero): the same guess, in each
  script, and it hides a real unknown angle on the control.
- A separate positioning block (rotate in place before the XY traverse):
  needs new script logic in each dialect. The zero already rides the first
  rapid at the clearance, above the stock, after the Z retract.
- Zero angles also on streams that never lean: changes the bytes of every
  upright program that states the Z column; no safety gain.

**Cost to change later.** Small. The rule is one match arm in `dump` and one
fill loop in the stamp. A new dialect with an angle memory gets the known
start for free.
