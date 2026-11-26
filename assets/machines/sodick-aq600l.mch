{
  "general": { "model": "Sodick AQ600L", "vendor": "Sodick", "kind": "wire_edm",
    "description": "Sodick AQ600L wire EDM with the LN2W control (#4045). Travels: X/Y 600 x 400 mm and U/V 80 x 80 mm, taper +-20 deg, from Sodick's AQ400L / AQ600L brochure (page 7) as the review of PR #4065 cited it - CHECK THEM against your machine's specification sheet; each travel is centred on zero. Stated by an owner of this machine (2026-11-25): automatic wire threading (T91 / T90), high flushing (T84) and submerged machining (T94) at each start. The LN2W post writes a constant taper as a 4-axis G74 program with TP / TN and U V on each move (#4075); set its post option for the part's bottom above the worktable. The angle form (G51 / G52) is not confirmed, so this file states no edm.taper_angle rule. EXAMPLE STARTER SET: the conditions C001 / C002 / C903 and the guide heights (0 and 100 mm, used only to check the U/V travel of a lean: 100 mm x tan 20 deg = 36.4 mm, inside the 40 mm half travel) are examples so that a first Wire Contour generates and posts - they are not Sodick figures; replace them with the rows of your control's condition table and your machine's guide heights before you cut. Wire diameters and codes have no public source and are left out" },
  "kinematics": { "default": {
    "units": { "length": "mm", "angle": "degrees" },
    "parts": [
      { "id": "base", "name": "table", "type": "table", "parts": [
        { "id": "x", "name": "X", "type": "linear", "travel": { "min": -300.0, "max": 300.0 }, "parts": [
          { "id": "y", "name": "Y", "type": "linear", "travel": { "min": -200.0, "max": 200.0 } }
        ] },
        { "id": "u", "name": "U", "type": "linear", "travel": { "min": -40.0, "max": 40.0 }, "parts": [
          { "id": "v", "name": "V", "type": "linear", "travel": { "min": -40.0, "max": 40.0 } }
        ] },
        { "id": "head", "name": "guides", "type": "head" }
      ] }
    ] } },
  "controller": { "post": "Sodick Wire EDM (LN2W)" },
  "edm": {
    "lower_guide_z": 0.0, "upper_guide_z": 100.0,
    "max_taper_deg": 20.0,
    "awt": true,
    "tank": "submerged", "flushing": "high", "skim_flushing": "low",
    "conditions": [
      { "id": "C001", "name": "First cut (example)", "role": "rough", "gap": 0.04, "wire_diameter": 0.25,
        "notes": "EXAMPLE - not a Sodick figure; replace with the row of your control's condition table" },
      { "id": "C002", "name": "Second cut (example)", "role": "skim", "gap": 0.02, "wire_diameter": 0.25,
        "notes": "EXAMPLE - not a Sodick figure; replace with the row of your control's condition table" },
      { "id": "C903", "name": "Third cut (example)", "role": "finish", "gap": 0.01, "wire_diameter": 0.25,
        "notes": "EXAMPLE - not a Sodick figure; replace with the row of your control's condition table" }
    ]
  }
}
