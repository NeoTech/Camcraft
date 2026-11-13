{
  "general": { "model": "SYNTHETIC WIRE EDM (FICTIONAL CODES)", "vendor": "Camcraft", "kind": "wire_edm" },
  "kinematics": { "default": {
    "units": { "length": "mm", "angle": "degrees" },
    "parts": [
      { "id": "base", "name": "table", "type": "table", "parts": [
        { "id": "x", "name": "X", "type": "linear", "parts": [
          { "id": "y", "name": "Y", "type": "linear" }
        ] },
        { "id": "head", "name": "guides", "type": "head" }
      ] }
    ] } },
  "edm": {
    "awt": true,
    "conditions": [
      { "id": "C110", "name": "synthetic rough", "gap": 0.04, "wire_diameter": 0.25, "role": "rough", "notes": "SYNTHETIC - invented numbers, no machine's manual" }
    ],
    "codes": {
      "condition": "FX-COND {id}",
      "comp_off": "FX-COMP-OFF",
      "thread": "FX-THREAD",
      "cut": "FX-CUT",
      "tip_disposal": "FX-TIP-OUT"
    }
  }
}
