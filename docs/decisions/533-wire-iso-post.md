## #533 - Generic ISO-style wire EDM post (codes as machine data)

The umbrella (#290) decided that dialects are data (D5): one generic post per EDM kind writes the RS-274 motion words itself and takes every EDM-specific code from the machine's `.mch` `edm.codes`, refusing by name when one is missing. #532 decided the path is the wire-centre path (kerf already in it) with no feed. These are the points this post settled.

**Decided:**

- **Compensation is cancelled, never engaged.** The Wire Contour's moves are the wire-centre path (#532 puts wire radius + gap in it). The post writes `edm.codes.comp_off` before every cut and never reads a left/right compensation code: a control offset on top of the CAM offset would double the kerf. `comp_off` is required for that reason - an unstated cancel could leave a previous program's offset active.
- **Required codes.** Always `condition` and `comp_off`. With `edm.awt: true` also `thread`, `cut` and `tip_disposal`. Each missing one refuses the post as "machine <m> states no edm.codes.<x>". Without AWT (false or unstated: not claimed) the post writes ISO 6983 `M00` with a comment for the operator to thread at the start hole and to cut the wire at the end, so the AWT codes are not needed.
- **The condition code is a template.** `edm.codes.condition` must hold `{id}`, which takes the op's condition id (`"FX-COND {id}"` -> `FX-COND C110`). A template with no `{id}` would select nothing and is refused by name. A condition id that is not one word (letters, digits, `.`, `_`, `-`) is refused.
- **Order in a section:** G0 to the start hole (the first cut's start, the threading point), thread, select the condition, cancel compensation, then the cut in X Y; after it, cut the wire and dispose of the tip. The program assumes the wire is NOT threaded at its start, and says so in its header comment. A rapid while threaded, a Z change (2-axis: one programmed plane), a posed move, an arc off the XY plane, a cycle, a tool change and a spindle speed are refused by name.
- **No F word.** The generator's condition sets the cutting rate (#532: the moves carry no feed).
- **Plain RS-274 / ISO 6983 words only:** `%` fences, G20/G21, G90, G17, the work-offset word, G0/G1/G2/G3 with incremental I J, M00/M01, M30. These were judged universal enough; the words D5 named as unsafe to default (G41/G42, G21/G22-style EDM meanings, threading and condition words) all come from `edm.codes`.
- **The engine's door: `wireEdm = true`.** The wire stream carries `PARAM EdmCondition`; a script that does not declare `wireEdm` is refused by name (as `liveTooling`, `threadHands`), so no cutting post can post a wire path as a cut, whatever the picker shows. The tool-less refusal in `op_streams` stays for any other tool-less kind.
- **Codes live on `Machine.edm_codes`, parsed by a new `edm_codes.rs`,** not inside #530's `Edm` struct: #530/#531 are still in review, and this keeps their files untouched. The mirror and `script_props` expose `machine.edm = { codes, awt }` and `machine.kind` for a wire or sinker EDM.
- **Registry.** "Generic ISO Wire EDM" in `BUILTINS`, the one `WIRE_POSTS` entry: it serves `wire_edm` only, is the default post for a wire EDM, and no cutting post serves a wire EDM.

**Why:** Prime Directive and D5 - a code the machine did not state is never guessed; the kerf is applied exactly once.

**Considered and dropped:**

- Posting the part contour with control compensation (G41/G42-style codes and the kerf in a register): #532 already decided the CAM path carries the kerf; doing it twice, or re-deriving the part contour, is wrong output.
- Defaulting M00/M30 from `edm.codes`: these are ISO 6983 program-control words, not EDM codes; requiring them would make every machine file restate the standard.
- Making `tip_disposal` optional under AWT: the issue lists it with the AWT codes, and an unstated disposal is not "none".
- Adding `codes` to `Edm` itself: the natural home, but it would edit files of two PRs still in rework. It can move there when they land (a field move, no file-format change).

**Not exercised on a real EDM control (D10):** the golden is posted against the synthetic fixture `crates/app/tests/edm/wire-iso-fixture.mch`, whose codes (`FX-...`) are invented. A user with a real machine fills `edm.codes` from its manual and dry-runs before any cut.
