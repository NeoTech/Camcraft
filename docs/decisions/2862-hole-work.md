## #2862 - Hole-work catalog case

**What was decided.** Add one original 90 x 90 x 20 mm plate with four
drill holes on a 60 mm bolt circle. Use a separate hole for Bore, Circular
and Thread. Add a tap-drill operation before the internal M8 x 1.25 thread.
Use the existing command scripts, catalog discovery and default Fanuc mill
post. Keep the model embedded in the saved project.

**Why.** One small case makes the four operation types easy to compare.
The posted program shows G81 cycles, hole milling arcs and the thread
helix. The prior tap drill gives the thread mill an open hole. An embedded
mesh lets the project open without a path relative to the script directory.
The multi-form thread mill tip follows the helix from Z8 to Z9.875. Its
axial cutting teeth cover the intended 12 mm thread length.

**Model and tool choices.** The original generator tiles annular cells and
checks their edge pairs. It uses 96 flats around each hole. The drill,
bore and circular envelopes have 0.2 mm radial clearance. The thread hole
keeps the M8 x 1.25 minor radius, 3.3235 mm, with its floor at Z4.
The script extends the drill and end-mill flutes to 25 mm. A custom
6.6 mm tap drill starts from the catalog drill. No holder is modelled.

**What was dropped.** Do not use automatic drilling cycle selection.
An explicit Drill cycle makes G81 stable when the hole dimensions change.
Do not use a 10 mm radius Circular cut with several radial passes in this
case. Verification refused that cut because of rapids through stock.
Helical entry also produced crossing warnings. Use a 5 mm radius cut with
a plunge entry; it needs one radial pass and verifies without warnings.
The separate Circular defect is #2898. It is linked from #2862 and the
catalog umbrella #2392.

**Limits.** The through-drill point reaches Z-2.803 and the bore reaches
Z-1. These moves require clearance below the plate. The report names the
break-through. The case has no fixture or spoil board. No machine or
control simulator has run the program. The model represents the thread
minor hole, not its teeth.

**Cost to change later.** A different hole size, tool, entry or cycle
requires a script change and a rebuild of the committed project and NC
program. A change to the model also requires a generator change. Update
the case pin and README when their stated dimensions or output change.
