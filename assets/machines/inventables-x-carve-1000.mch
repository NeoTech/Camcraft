{
  "general": { "model": "Inventables X-Carve 2021 1000 mm", "vendor": "Camcraft", "kind": "router" },
  "kinematics": { "default": {
    "units": { "length": "mm", "angle": "degrees" },
    "parts": [
      { "id": "base", "name": "table", "type": "table", "parts": [
        { "id": "col", "name": "Z", "type": "linear", "travel": { "min": -114.3, "max": 114.3 }, "parts": [
          { "id": "ram", "name": "Y", "type": "linear", "travel": { "min": -749.3, "max": 749.3 }, "parts": [
            { "id": "sld", "name": "X", "type": "linear", "travel": { "min": -749.3, "max": 749.3 } }
          ] }
        ] },
        { "id": "head", "name": "head", "type": "head",
          "spindle": { "min_speed": 10000.0, "max_speed": 30000.0, "power": 0.8 } }
      ] }
    ] } },
  "controller": { "post": "Grbl 1.1", "synced_configuration": { "parts": {
    "X": { "coordinate": 0 },
    "Y": { "coordinate": 1 },
    "Z": { "coordinate": 2 }
  } } }
}
