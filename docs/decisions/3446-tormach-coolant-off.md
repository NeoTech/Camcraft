## #3446 - The Tormach PathPilot mill post reads the operation's coolant

**Decided:** `assets/posts/tormach.cps` declares `coolantModes = true` and
reads `PARAM Coolant`, as `linuxcnc.cps` already does. Off writes no
coolant word. Mist, Air and Through tool take the machine's own entry of
that family. An ask that the machine does not state is refused by name,
with the operation name. Flood keeps its old law: the machine's Flood
code, silent on a machine with no coolant list.

**Why:** Before, the engine refused every non-Flood ask on this post,
because the post did not read the ask. On a dry router (Tormach 24R, no
coolant list) the user had to pick Flood to post. Flood and Off then gave
the same G-code, so the refusal blocked a correct choice. Reading the ask
in the post makes Off the honest choice, and lets a 770MX post its Mist M7.

**Dropped:**
- An engine exception: let Off through when the machine has no coded
  coolant. Dropped because the engine cannot know what a post writes for
  Flood, and it keeps a non-reading post that writes a fixed M8.
- Only Off in the post (no Mist or Air). Dropped because the lookup is
  small and the same as the other posts; half a reader would need the
  engine refusal again for the other asks.

**Cost to change later:** low. One post script and its tests
(`crates/post/tests/tormach_coolant.rs`, `crates/app/tests/coolant_air.rs`).
The flood refusal text on a machine with only non-flood coolants now says
"pick another coolant for the operation (Off cuts dry)", as `linuxcnc.cps`
says.
