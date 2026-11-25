## #1869 - Engrave and Trace follow placed DXF sketch strokes

**What was decided.** Engrave and Trace take a whole profile, or selected
chains and layers. They trace the placed centerlines at each cut level.
The motion keeps straight segments and directed circular arcs. A full
circle is divided into arcs with distinct ends. A tolerance join can
group two curves whose ends are apart. The tool lifts and re-enters at
that gap to keep both curves as drawn. The operation's climb setting
selects the canonical direction or reverses the chain. Reversal changes
the order and both ends of every segment, and flips each arc's sweep.
The project and MCP use the existing profile references and operation
parameters.

**Why.** A stroke may be open and has no bounded pocket or side to
offset. A circular stroke already states its center and sweep. They
survive sketch placement and rotation of the work offset. Separate
plunges and retracts keep links between strokes clear of stock. On a
setup with a profile and stock, the cut still has a physical surface and
underside. The mark law reads that stock, limits the cut depth, and
checks the material beneath it. The same verification and posting gates
check crossings, rapids through stock and fixture collisions. When the
setup also has parts, their existing mark checks apply.

**Dropped.** Converting the strokes to closed regions would join open
ends. Joining nearby curve ends with a new feed would add a cut across a
gap. The maintainer's first-party Spirit comparison illustrated reversing
the sequence and each segment together. Its nearest-end routing can
reverse an open stroke to save travel. CamCraft keeps the direction selected
by its own climb setting and uses the existing chain order.

**Cost to change later.** A different ordering policy needs to preserve
the selected direction, the stock and fixture checks, and the separation
between strokes. A per-chain direction control would need stored
overrides and command and MCP support. Neither is necessary to store or
reopen the paths in this change.
