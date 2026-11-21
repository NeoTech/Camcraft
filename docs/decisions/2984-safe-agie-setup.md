## #2984 - Encode one Agie setup without erosion steps

Provide a Rust API that encodes the SBL piece script and its SBR import manifest for an existing nominal ISO. Use one closed punch, 20 mm steel 0001, Berco Cut 0.25 and the measured Ra 0.30 request. The stock and 2 mm threading hole use integer micrometres. Derive all names from one validated file stem.

A controller can bind a quality name to stale data after a successful assignment. Clear the work's erosion steps during import. Return the expected control settings and offsets for the operator to check before release.

Main has no native Agie posting path. The separate encoder lets a caller create and inspect these records before that path ships. The first workspace writer must use the guarded encoder and retain its own contour, stock and lead checks. A generic post filter could miss a writer that publishes its own files. Extending this API to more work types or conditions requires measured offsets, geometry checks in the caller and control verification.
