## #4095 - Heidenhain Klartext post: smoothing as cycle 32 TOLERANCE

**What was decided.**

- `heidenhain-klartext.cps` writes a section's smoothing ask (`PARAM
  Smoothing` / `SmoothingMode`, #616) as cycle 32 TOLERANCE. Before this
  change, the post refused each smoothing ask.
- The blocks come after the section's `M3` and `M8`/`M9` and before its
  first move:
  - `CYCL DEF 32.0 TOLERANCE`
  - `CYCL DEF 32.1 T<mm>`
  - `CYCL DEF 32.2 HSC-MODE:<0|1> TA0`, only where the control takes the
    mode word. `0` is finish and `1` is rough. `TA` is the rotary-axis
    tolerance. This post moves no rotary axis, so it writes `TA0`.
- The post reads the two resolver flags of #4226, which the DIN/ISO post
  reads for `G62`:
  - `g62Mode`: `yes` writes `32.2`; `option2` writes it only when
    `installed_options` lists option 2; `no` never writes it.
  - `g62TMaxUm`: the largest T on the 1 µm grid. 0 means no cycle 32.
- T is written on the 1 µm grid, as the DIN/ISO post does. These asks are
  refused by name:
  - a T above the largest T of the profile;
  - a positive T that rounds to 0 (the control reads T 0 as the machine
    builder's tolerance);
  - a profile with no cycle 32;
  - a rough ask where `32.2` cannot be written;
  - a mode with no tolerance, a mode that is not finish or rough, and a
    tolerance that is not a positive number.
- The section end resets the cycle: `CYCL DEF 32.0 TOLERANCE` and
  `CYCL DEF 32.1` (no T). Thus no tolerance goes into the next operation.

**Fehlmann Picomax 55 (iTNC 530, NC 340490-03, profile itnc-49x).** The
machine file lists no installed options, so a finish ask posts `32.0` and
`32.1` only, and a rough ask is refused. The largest T is 10 mm (the
narrowest documented range, #4226). A program with no smoothing ask is
byte-identical: the `picomax.nc` and `picomax_cycles.nc` goldens did not
change.

**Sources.** iTNC 530 Klartext user manual 533 190-22 (NC 340 49x-03),
section 8.10 "TOLERANCE (Cycle 32)", pages 509-511:

- the example blocks `CYCL DEF 32.0 TOLERANCE`, `CYCL DEF 32.1 T0.05`,
  `CYCL DEF 32.2 HSC-MODE:1 TA5`;
- "Finishing=0, Roughing=1";
- the cycle is DEF active;
- a cycle 32 with T only gets the two other parameters with the value 0;
- the reset: "Redefine it and confirm the dialog question for the
  tolerance value with NO ENT";
- "The HSC MODE and T parameters are only available if on your machine
  you have software option 2 active (HSC machining)". The ISO manual of
  the same software says this of `P01` and `P02` (#4226). Thus "T" here
  is the rotary tolerance `TA`, and the tolerance value itself needs no
  option.

**Inferred, not printed.** The manual prints no block for the reset. The
post writes `CYCL DEF 32.1` with no T: the NO ENT answer leaves the
tolerance empty. This is the same inference as the bare `G62` reset of
the DIN/ISO post. If the control reads the empty block as T 0, the
result is the same: the tolerance of the machine parameter.

**Considered and dropped.**

- Write T only and give a warning for a rough ask (the DIN/ISO post does
  this). The Klartext post sets `failOnWarning`, so a warning stops the
  post too. A refusal with the cause is clearer.
- Write `32.2 HSC-MODE` on the Picomax: the option 2 note says the
  control may not take it.
- Four decimals for T (0.1 µm input): the flag `g62TMaxUm` and the
  DIN/ISO post use the 1 µm grid. One grid for both posts keeps the range
  check the same.
- No reset at the section end: a tolerance of a finish operation would go
  into the next operation.

**Cost to change later.** The words are in one function pair
(`smoothOn` / `smoothOff`) at the end of the post. A machine file that
lists option 2 gets `32.2` with no code change.
