{
  "general": { "model": "Test VMC tree", "vendor": "Camcraft", "kind": "mill" },
  "kinematics": { "default": {
    "units": { "length": "mm", "angle": "degrees" },
    "parts": [
      { "id": "base", "name": "table", "type": "table", "parts": [
        { "id": "x", "name": "X", "type": "linear", "travel": { "min": -300.0, "max": 300.0 }, "parts": [
          { "id": "y", "name": "Y", "type": "linear", "travel": { "min": -200.0, "max": 200.0 } }
        ] },
        { "id": "z", "name": "Z", "type": "linear", "travel": { "min": -100.0, "max": 400.0 }, "parts": [
          { "id": "head", "name": "head", "type": "head",
            "spindle": { "min_speed": 100.0, "max_speed": 12000.0, "body_diameter": 100.0, "body_length": 150.0 },
            "tool_station": { "max_tool_length": 100.0, "max_tool_diameter": 20.0 } }
        ] }
      ] }
    ] } },
  "controller": { "post": "Fanuc", "synced_configuration": { "parts": {
    "X": { "coordinate": 0, "max_normal_speed": 5000.0, "max_rapid_speed": 10000.0 },
    "Y": { "coordinate": 1, "max_normal_speed": 5000.0, "max_rapid_speed": 10000.0 },
    "Z": { "coordinate": 2, "max_normal_speed": 3000.0, "max_rapid_speed": 8000.0 }
  } } }
}
