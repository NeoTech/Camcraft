## #2694 - Wire input preimages

**What was decided:** Detect healing when an input edge disappears, after
removing redundant collinear vertices. Match surviving offset edges by
direction and midpoint. Project their vertices to input arc coordinates.
Hold each excised span at its trim vertex. `Ring::at` and `Ring::nearest`
use those coordinate spans. When either plane heals, sew both planes on
their input coordinates. Preserve the existing offset-length rule otherwise.

Use nearby surviving offset segments for widening directions on healed
rings. Include tiny bevels within one offset distance to prevent reversal.
Keep cover, folds, crossing, lean, and 33-height clearance against the
unhealed product. Refuse unknown or non-monotone source matches by name.

**Why:** Shortened offset lengths caused phase drift and missing fans.
Normals from removed input edges caused a widening fold. Eleven pins,
the 12-step quick gate, and clippy pass. No golden files change.

**Considered and dropped:** A vertex-count check missed healing after
straight-edge subdivision. Direct pairing of shortened rings failed the fan
pins. Original-edge widening failed its mutation check. Relaxing cover is
separate work. Changing all unhealed pairings would change established behavior.

**Cost to change later:** Broader join support needs new fixtures and source
matching rules in the wire module. This change adds no dependency, file
format, or command.
