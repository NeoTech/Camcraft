## #2977 - Two through-openings of one retained plate in one native wire EDM job

**Decided.** Match each selected rim to one through-opening of the same modeled plate. Require exactly two openings, one for each operation, and no other model or placed drawing. Keep each work's threading hole, inward cut side, capture and 1 mm separation. Require a clear band along X or Y between the two cutting areas: the opening, its hole and 3 mm lead, the rough corner and the wire radius. Refuse a punch and a die in one group.

**Why.** The two operations protect the same retained material. Matching each selected boundary against the complete vertical prism checks that the cut goes through the plate. Disjoint cutting areas require the operations to use different openings and keep the plate between them uncut. The offline Agie control check confirmed this layout: `CCB030[4]` and the die side on both works, no formal error, and no cutting move across the plate between the openings.

**Port to the zero-step setup (#2984).** The die group uses the setup records of the control-checked punch group (#2961) with `ID_PUNCH,c_die` on each work (`PunchGroup20::encode_die`). The punch group bytes do not change. The group keeps the one CCB030 quality object of #2984. The control result of the original writer was not reused: the ported files were imported again on the emulated control, and those bytes are the golden files.

**Alternatives and cost.** A mixed punch and die group needs interference checks of the punch against the whole retained plate, and no control run covers it. More openings need one-to-one rim matching, a clearance check between each pair and a check of the control's release sequence (#2980). A CCB035 die group has no control check and stays refused.
