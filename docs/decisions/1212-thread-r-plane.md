## #1212 - Thread milling starts its feed at the operation's R plane

**What was decided**

1. **The kernel takes the R plane as one optional height.**
   `ThreadParams::r_z: Option<f64>`; the app passes the resolved
   `top_z` ("R plane (feed starts here)"). Every feature's rapid stops at
   `max(R, the ride's run-out)` and the feed down the axis (or beside the
   boss) starts there; the feed back out climbs to the same height
   before the link rapid. `None` is the #894 ride (each feature's own
   top), which keeps every kernel golden (#894, #985, #988)
   byte-identical without touching them.
2. **The run-out still wins when it stands higher.** The #894 ride rapids
   to `max(top, run-out)`: a rising helix's last lead-out ends above the
   mouth (1.75 mm in the internal golden). An R plane under that height
   would leave the ride's own last move above the rapid's end, so the
   rapid stops at the run-out and the feed starts from there - higher,
   never lower, than the stated R.
3. **Never below a feature's top, never above the link plane - refused
   by name.** In the kernel (any reference, at generate) and at the door
   for a Fixed `top_z` (create, edit, retool; a referenced height
   resolves only with the world). The door's R check runs after the
   tool/feature laws so their refusals keep their names.
4. **The stock law.** At generate, before the way-in laws (#992/#985),
   every vertical rapid down in the thread path is read in the prior
   grid over the mill's full disc (`verify::prior_surface`, the reading
   the rapid law and the drill R-plane law use); stock standing above
   the rapid's end is refused naming the R plane, the stock height and
   the fix. Unknown prior / unreadable disc is left to the way-in laws,
   which refuse both by name.
5. **How defaults are seeded: unchanged.** A new Thread op's `top_z` is
   the carried top (Top of Model by default), as before. Every thread
   golden's fixture already states `top_z` equal to the features' top,
   so no golden moved. Two MCP fixtures typed a hole/boss mouth at
   z 20 over a 15 mm block (R = top of model = 15); they now sit on the
   block's top, since R under the mouth is refused by design. One
   boss door fixture states its R on its taller boss's top so the reach
   refusal it pins is still the one reported.

**Why**

The Heights tab shows the R plane as where the feed starts; the ride
ignored it. A drill cycle honours it; a thread mill should too, and a
plane under a feature's top is a rapid into material.

**Considered and dropped**

- *Clamping R up to each feature's top* (silent): the Prime Directive
  says refuse, and the issue asks for it.
- *Seeding `top_z` from the features' highest entry*: changes what the
  page shows for every new op and needs the holes before they exist
  (they are detected after creation).
- *Rapiding to R even under the run-out*: kept the #894 ride's
  `max(top, run-out)` shape so an R on the top changes nothing; the
  ride's lead-out would rise above the rapid's end anyway.

**Sources (public)**

RS-274 / ISO 6983 canned cycles (rapid to R, feed from R); Machinery's
Handbook / Smid, "CNC Programming Handbook" (never rapid into uncut
stock). No HSMWorks / Fusion material consulted.
