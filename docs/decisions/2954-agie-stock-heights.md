## #2954 - Qualify discrete Agie QN2 punch heights

Accept 15, 20, 40 and 80 mm stock for the existing cold die steel and Berco Cut 0.25 mm recipe. Each thickness has its own measured maximum wire-centre offset. The user guide records all four pass offsets from the rectangular punch. No intermediate height can inherit a measurement from a neighbouring height. The controller generates its own passes and applies its own compensation.

Use the measured rough offset for the corner miter. Round that offset plus the wire radius upwards to a tenth of a millimetre for the straight-face allowance. A common 0.4 mm allowance alone misses the extra reach at an acute corner. Retain the stock-top work datum, checks on both planes of the workpiece and both height fields in the controller job.

The reference jobs and the direct CamCraft jobs use the same six quality parameters. The reference exporter assigns a sequential quality name, while CamCraft names it QN2. Qualification requires the native CamCraft job to pass the control’s formal check and reproduce the measured offsets in its graphical check. This evidence applies to this recipe and simulated control installation.
