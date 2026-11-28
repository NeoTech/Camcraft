# Fit arcs on closed model chamfer passes

The closed generator already provides the nominal circuit, travel direction,
and selected material side. Use the same sided fit as other planar toolpaths
at the lower of the operation tolerance and the usual fit cap. The normal
verification and post see the resulting moves. The cone ride and open
model-edge generator keep their original depth and linking rules. Invalidate
saved chamfer caches once so they regenerate and verify the fitted paths.
