## #2482 - Mach3 Mill post (3-axis)

Sources: the public "Using Mach3Mill" manual (the G-code and M-code lists
and the General Config notes), behaviour only, and RS-274 / ISO 6983 for
the shared words. The script is derived from our own `grblhal.cps`. No
vendor post or vendor output was consulted.

- **Decided: tool change is `T<n> M6`, then `G43 H<n>`.** Z goes up first.
  **Why:** Mach3 keeps a tool table, and M6 runs the user's M6Start /
  M6End macros (a manual stop, a tool setter or an ATC). The post cannot
  know what the macro does, so it lifts Z before it.
- **Decided: canned cycles G73, G81, G82, G83, G85, G86, G89 with G98.**
  Mach3 lists these cycles. A dwell P on G82, G86 and G89 is in seconds.
  A G83 dwell is refused by name, because G83 has no P word in the list.
- **Decided: the dwell unit is a post option (`dwellInMilliseconds`, off by
  default).** **Why:** Mach3 has the General Config setting "G04 Dwell
  param in ms". The post cannot read the control's setting, so the user
  states it. Off writes seconds.
- **Decided: G41/G42, G84 tapping and probing are refused by name.** Mach3
  has G41/G42, but they depend on the control's tool table radius, which
  the post does not know. Tapping and probing depend on the user's
  machine setup and macros.
- **Decided: G91.1 in the header.** Arc centres are incremental, stated
  every time. Mach3's IJ mode is a setting, so the post never assumes it.
- **No own machine.** A Mach3 user picks an existing 3-axis machine (for
  example "VMC 3-axis") and edits the limits.

Not exercised: no reading on a real Mach3 control. That Mach3 accepts
G91.1 is from the manual's word list only, not tested on a control.

Cost to change later: low. The script is one asset and one registry line.
