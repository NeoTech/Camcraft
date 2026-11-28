{
  "general": { "model": "Inventables X-Carve Pro 4x4", "vendor": "Camcraft", "kind": "router" },
  "kinematics": { "default": {
    "units": { "length": "mm", "angle": "degrees" },
    "parts": [
      { "id": "base", "name": "table", "type": "table", "parts": [
        { "id": "col", "name": "Z", "type": "linear", "travel": { "min": -101.6, "max": 101.6 }, "parts": [
          { "id": "ram", "name": "Y", "type": "linear", "travel": { "min": -1219.2, "max": 1219.2 }, "parts": [
            { "id": "sld", "name": "X", "type": "linear", "travel": { "min": -1219.2, "max": 1219.2 } }
          ] }
        ] },
        { "id": "head", "name": "head", "type": "head",
          "spindle": { "min_speed": 8000.0, "max_speed": 24000.0, "power": 1.5 } }
      ] }
    ] } },
  "controller": { "post": "Grbl 1.1", "synced_configuration": { "parts": {
    "X": { "coordinate": 0 },
    "Y": { "coordinate": 1 },
    "Z": { "coordinate": 2 }
  } } }
}
