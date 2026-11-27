## #2806 - Distinct rotary operation icons

**Decision.** Draw original glyphs on the existing 24-unit grid. Rotary
Contour shows an upright cylindrical part, two waterline rings, and an
upward axis arrow. Rotary Parallel shows a horizontal cylindrical part,
two longitudinal passes, and a rightward axis arrow. Keep the existing
asset names, registry entries, toolbar layout, and operation behaviour.
Generate the 32x32 PNGs with `cargo xtask icons`.

**Why.** The part silhouettes and arrows distinguish the Z and X axes at
16px. The pass marks provide a second distinction. The glyphs use the
existing round strokes and need no new asset source or licence notice.

**Considered and dropped.** Changing only the internal line pattern keeps
the similar silhouette reported in the issue. Axis letters and a cutter
would add small details that compete with the part at 16px. Colour alone
would not distinguish the monochrome icons in both themes.

**Cost to change later.** Edit the two SVG masters and regenerate their
PNGs. No code, layout, or project-file change is required.
