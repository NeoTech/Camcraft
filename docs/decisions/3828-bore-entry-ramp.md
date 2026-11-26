## #3828 - Bore entry: plunge, ramp or helix, capped by the approach angles

**What was decided.** The Bore operation gets an `entry` field (plunge, ramp, helix). Plunge is the
default and is not written to the project file. The links' approach `ramp_angle` and `helix_angle`
cap every descent. A bore that chose ramp or helix may use up to 4096 ring-ramp legs
(`MAX_BORE_ZIG_LEGS`). A plunge bore keeps the 64-leg limit. Arc is refused by name.

**Why.** A hole barely wider than the cutter has a ring of a few hundredths of a millimetre. A 5 degree
ramp needs about 73 legs per millimetre of stepdown there, and the old 64-leg limit made the ramp
give up and dive axially.

**Dropped.** Raising the global 64-leg limit: it would let a cutter that may not plunge into solid
ramp where it is refused by name today (#1036, pinned by `bore_entry`). A spiral descent variant:
more kernel code for the same path.

**Cost to change later.** The leg limit is one constant. The `entry` field is additive in the file.
