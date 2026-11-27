{
  "general": {
    "model": "3040 router (FluidNC)",
    "vendor": "Camcraft",
    "kind": "mill",
    "description": "A generic 3040-class hobby router on a FluidNC controller (#3192). TYPICAL VALUES, NOT A MAKER'S SHEET: the owner confirms each one for the real machine. About 300 x 400 mm work area (X 300, Y 400), Z 70 mm, an 800 W spindle on a VFD (6000-24000 rpm), manual collet tool change, no tool changer, no coolant, no homing assumed (no safe_retract). Pairs with the FluidNC post."
  },
  "kinematics": { "default": {
    "units": { "length": "mm", "angle": "degrees" },
    "parts": [
      { "id": "base", "name": "table", "type": "table", "parts": [
        { "id": "col", "name": "Z", "type": "linear", "travel": { "min": -70.0, "max": 70.0 }, "parts": [
          { "id": "ram", "name": "Y", "type": "linear", "travel": { "min": -400.0, "max": 400.0 }, "parts": [
            { "id": "sld", "name": "X", "type": "linear", "travel": { "min": -300.0, "max": 300.0 } }
          ] }
        ] },
        { "id": "head", "name": "head", "type": "head",
          "spindle": { "min_speed": 6000.0, "max_speed": 24000.0, "power": 0.8 } }
      ] }
    ] } },
  "controller": { "post": "FluidNC", "synced_configuration": { "parts": {
    "X": { "coordinate": 0, "max_normal_speed": 2000.0, "max_rapid_speed": 2500.0 },
    "Y": { "coordinate": 1, "max_normal_speed": 2000.0, "max_rapid_speed": 2500.0 },
    "Z": { "coordinate": 2, "max_normal_speed": 800.0, "max_rapid_speed": 1000.0 }
  } } }
}
