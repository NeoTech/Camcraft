## #2343 - Rotary Parallel leaves the end face of a held end

Sources: chuck and tailstock practice from machine-tool handbooks; our own
raster, link and stock code. No Fusion or HSMWorks material was consulted.

- **Decided: Rotary Parallel reads the setup stock and leaves the end face of
  a held end.** Where the stock goes on past an end of the model along the
  fitted axis by more than one tool diameter (plus the tolerance), that end is
  a held end: the bar goes on into a chuck or onto a centre. At a held end the
  centre of the ball does not go past the end plane of the model. The
  operation adds a note that names the end and the distance. A free end, where
  the stock ends with the model, is cut as before. **Why:** the end-face pass
  at a held end is a slot one tool diameter wide from the skin of the bar to
  its axis. That parts the bar. The post blocked it only for a tool with a
  short flute. The stock already says where the bar goes on, so the generator
  can know this without a new setting.

- **Decided: the limit is one tool diameter.** Stock that goes on by one tool
  diameter or less is a facing allowance. The end-face pass removes all of it
  and leaves no disc on the bar. That case does not change. More than one
  diameter leaves a disc that the pass cuts off, so it is a held end.

- **Decided: the centre of the ball stops on the end plane.** The turned
  surface is then finished to its end, and the verification reports no rest
  there. Half of the ball goes past the plane. There it cuts no deeper than
  the surface that it rides on, so the bar past the model keeps the radius of
  the model's end or more. **Dropped:** the centre one radius inside the
  plane, so that the ball never crosses it. I built this first. It left a
  strip one tool radius wide with the fillet of the ball, and the example got
  a scallop warning of 0.749 mm against a gate of 0.520 mm. The model was not
  finished to its tolerance, and the user had no way to finish it.

- **Decided: a sloped entry does not start past a held end.** With the
  sloped-entry law (a tool that is not centre-cutting), a ramp whose top is
  past the plane is not used. That infeed stays a radial plunge, and the
  centre-cutting guard judges it as before.

- **Decided: a model with no length between two held ends is refused by
  name.** No pass fits there.

- **Dropped: a new setting** (an axial window with two stations, or "leave
  the end faces"). It needs a parameter, a project-file version, a UI field
  and an MCP field. It is a `feat` and waits for the maintainer. With it, a
  user could also keep the pass off a free end face, or cut a held one.

- **Dropped: a refusal at each held end.** It makes the job impossible on a
  held bar, which is the usual way to hold rotary work.

- **Dropped: a warning only.** The program would still part the bar.

- **Not changed: Rotary Contour and Rotary Offset.** They read their heights
  as stations and already have an axial window.

- **Limit.** The generator knows only the stock that the setup states. If the
  stock ends where the model ends but the real bar goes on (the kit job of
  #2342), CamCraft reads air there and cuts the end face as before. The guide
  pages say this. A model with a domed or coned held end is still cut on that
  dome or cone, up to the end plane.

- **The #1904 example.** The example project and program are generated again.
  The program has no feed move past X-1.25. A 1/2 in flute now posts, because
  the shank refusal came from the cut that is gone. The guide page drops its
  "air cut only" limit and keeps the air cut as the first run.

- **Cost to change later.** A setting can replace the stock reading: `held`
  then returns the window from the two stations. `clip` and the callers stay.
