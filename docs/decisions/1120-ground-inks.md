## #1120 - View cube, marker and axis inks become theme tokens (second family)

**What was decided.** Twelve colour tokens join `Tokens`, each also a
theme-file key (`colors.<name>`):
- the view cube and its nav buttons: `cube_edge`, `cube_label`,
  `cube_glyph_on`, `cube_glyph_off`;
- the corner hint: `viewport_hint`;
- the playback tool marker: `marker_stem`, `marker_ring`, `marker_dot`;
- the axis triad of the Axes button and the handles: `axis_x`, `axis_y`,
  `axis_z`;
- the hover gold of the view cube dot and of the machined facet:
  `viewport_hover`.

The dark table holds the old dark-ground set and the light table the old
light-ground set, so no pixel changes. `ground_ink::cube_ink_on` builds the
`CubeInk` from a token table; `cube_ink(light)` keeps its signature and
reads the built-in table of that ground. The face and button greys
(`face_base`, `face_span`, `hover_lift`, `button`) stay numbers chosen by the
ground: they are shading spans, not inks.

**The ground rule.** Before #1120 the cube chose its set by the lightness of
the viewport ground, not by the theme's base. A theme file on the dark base
with a light `viewport` got the light cube. To keep that, `apply_overlay`
takes the default of each of the twelve keys from the built-in table of the
resulting ground when that ground's lightness differs from the base's. A key
that the file sets always wins. The user guide says so.

The veil over a hovered facet is `ground_ink::facet_veil(viewport_hover)`:
the token's colour, read unmultiplied, at alpha 90. `Color32` stores its
channels multiplied by alpha, so reading `r()`, `g()`, `b()` darkened a
theme's semi-transparent hover a second time (found in the #3993 review).

**Why.** Flat names, as in the first family (`docs/decisions/1120-handle-inks.md`).
The ground rule keeps the behaviour of every theme file that exists, and it
is one `match` in one function.

**Considered and dropped.**
- Two sets of tokens (a dark-ground and a light-ground key for each ink).
  It keeps the old switch inside the draw code, but doubles the keys, and a
  theme author must set both to change one colour.
- Following the base only. It is simpler, but a dark-base theme with a light
  viewport would get white cube labels on a pale ground: a visible
  regression.
- Taking the shell's hover gold into family 1c. The token is the same ink as
  the view cube dot, so both sites move here; the `shell.rs` row goes from 9
  to 7.

**Cost to change later.** Small: twelve fields, twelve keys, one `match`.
