## #4434 - Built-in machine: Makera Z1 Pro

**Decision:** Ship `assets/machines/makera-z1-pro.mch` as a copy of
`makera-z1.mch` with two changes: the model name, and a feed and rapid
ceiling of 2000 mm/min on X, Y and Z. The file names the shared
"Makera Carvera" post. No new post.

**Why:** A machine file holds the travel, the spindle, the feed limits
and the coolants. The post holds the G-code dialect. Makera's public Z1
pages give the same 200 x 200 x 100 mm envelope and the same 150 W /
13000 rpm spindle for the Z1 and the Z1 Pro. The Pro has closed-loop
steppers and ball screws, with a 2000 mm/min maximum against the Z1's
1200 mm/min. The user, who owns a Z1 Pro, states that all Makera machines
use the same G-code commands now. So only the limits differ.

**Considered and dropped:**
- A Z1-specific post. The dialect is the same, so a second post would only
  duplicate `carvera.cps`.
- The vendor Fusion and FreeCAD posts as a source. The Clean-Room Protocol
  forbids the text of a vendor post, and `carvera.cps` uses no vendor post
  text.

**Cost to change later:** If a Z1-family firmware diverges from the
Carvera dialect, add a post and change the `controller.post` line of the
two Z1 files. The test `crates/app/tests/makera_z1.rs` covers the two
models through one `MODELS` table.
