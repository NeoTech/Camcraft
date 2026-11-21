## #3179 - Export one selected die opening for inspection

The operator can create a nominal inspection operation on the Classic 2S template with an empty machining condition. It stores the selected chain and the operator’s start point. It survives undo, redo and project reload. Ordinary generation still requires a valid resident condition.

The operator can export the nominal contour of one saved Wire Contour to a new Classic 2S inspection piece. The ISO preserves the LINE and ARC spans of the selected DXF chain in the setup work frame. Its SBL imports one work, a 3 mm threading hole and zero working steps. The receipt identifies the chain and the number of nominal spans.

The inspection piece uses zero native length and width for unknown physical stock XY. An offline control saved and reopened those values. The project box supports drawing placement and the 10 mm height. It supplies no claim about the physical stock footprint.

The inspection piece requests the reference settings for 10 mm steel 0001 and Berco Cut 0.25. An offline control displayed the QREF10 reference quality and zero steps. The effective erosion offsets, installed travel, workholding and entry and exit paths still need separate verification. The existing machining post retains its verification requirements. The export does not use a generated wire centreline or infer a slug catcher.

The inspection action checks the entire threading disk and the nominal contour against the other drawing primitives. A retraced line on the same layer belongs to the selection only when the chainer identifies its original and both endpoints match exactly. The export rounds ISO coordinates to five decimals and checks the resulting contour again. A synthetic R2 corner with a 0.000005038 mm difference between the rounded endpoint radii registered as a closed CCW die contour in the offline control. Larger discrepancies are refused. All three files become visible together in a new directory.

The mounted machine's X and Y limits apply in the selected work frame. The export checks analytic arc extrema before and after coordinate rounding. The native format used here cannot carry reversed axis polarity. An unspecified travel limit stays unverified. The controller displays a perpendicular lead default and the nominal Z datum. Neither establishes the physical stock position or a verified lead trajectory.

Machining this geometry would require verified leads, effective resident technology and physical slug retention. Those inputs belong in the cutting workflow.
