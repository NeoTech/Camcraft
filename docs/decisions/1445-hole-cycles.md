## #1445 - Hole cycles, first slice: the ream (G85)

Sources: RS-274/NGC's canned cycles (NIST IR 6556, The NIST RS274NGC
Interpreter v3) and LinuxCNC's G-code reference - G85 feeds to the bottom
and feeds back out to R with no dwell. No vendor post, manual text or
output was read.

**Ream is a new cycle kind, not a reuse of Back bore.** Back bore (G89)
already feeds out, but it is a boring cycle with a dwell word and feeds
out at the cutting feed. A ream needs its own name so the post can write
G85 and nothing else; a ream must never reach a control as a G89 (or a
G81/G86) because the control would then dwell, stop the spindle or rapid
over the reamed wall. Considered: a "feed out" toggle on Bore - dropped,
it hides the cycle's identity from the post and the setup sheet.

**One feed for both directions.** The chain feeds in and out at the
plunge feed, because G85's single F word drives both moves. The verifier
and stock simulation read the same chain the control runs. Considered:
the cutting feed for the retract (as Back bore does) - dropped, the
canned cycle cannot post two feeds, so the chain would lie about the
machine.

**No dwell on a ream.** G85 has no P word. A dwell typed beside a ream is
dropped from the cycle record, so no post writes one and no post has to
guess a dwell unit. Dwell-at-depth with a feed out (G89 with P) is a
follow-up.

**Ream is never Auto.** Auto cannot know a reamer from a drill by kind;
like Spot, it is the user's word.

**Ream rides the bore door.** It opens a pre-drilled hole: the tool must
fit the hole (refused by name otherwise), a stepped hole is refused (the
ring would be missed), and the centre-cutting law does not apply. Radial
(cross) reaming is refused by name, like radial boring.

**Canned posts refuse an unknown cycle by name.** Every post that maps
cycle words now ends its map with an `else` that refuses a cycle it does
not know ("canned cycle X is not known to this post - refused, not posted
as another cycle"), instead of falling through to the last branch. Today
the stream parser already rejects unknown kinds, so this is defence in
depth for the next cycle kind added; it is not reachable by a test.

**Posts without a canned ream expand it.** Heidenhain, TinyG and Carvera
post the explicit feed-in / feed-out chain. Sinumerik keeps its existing
ISO-dialect cycle words (G85), as it does for its other cycles.

**Fine boring (G76) is not in this slice.** It needs a spindle-orientation
capability and a user-given shift direction and amount; a default guess
could drag the tip across the bore. Deferred to its own issue rather than
guessed.

**Goldens are mapping records.** `crates/post/tests/golden/ream/*.nc` pin
our mapping, not a reading taken on a control; no output was run on a
machine.

Cost to change later: a new `CycleKind` touches the CL text, the stream
word, every post's map and the project spelling (`"ream"`); renaming it
would break saved projects.
