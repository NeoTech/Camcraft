{
  "general": { "model": "Carbide 3D Shapeoko 5 Pro 2x2", "vendor": "Camcraft", "kind": "router" },
  "kinematics": { "default": {
    "units": { "length": "mm", "angle": "degrees" },
    "parts": [
      { "id": "base", "name": "table", "type": "table", "parts": [
        { "id": "col", "name": "Z", "type": "linear", "travel": { "min": -155.0, "max": 155.0 }, "parts": [
          { "id": "ram", "name": "Y", "type": "linear", "travel": { "min": -623.0, "max": 623.0 }, "parts": [
            { "id": "sld", "name": "X", "type": "linear", "travel": { "min": -623.0, "max": 623.0 } }
          ] }
        ] },
        { "id": "head", "name": "head", "type": "head" }
      ] }
    ] } },
  "controller": { "post": "Grbl 1.1", "synced_configuration": { "parts": {
    "X": { "coordinate": 0 },
    "Y": { "coordinate": 1 },
    "Z": { "coordinate": 2 }
  } } }
}
