{
  "general": { "model": "Hurco VM10i", "vendor": "Camcraft", "kind": "mill" },
  "kinematics": { "default": {
    "units": { "length": "mm", "angle": "degrees" },
    "parts": [
      { "id": "base", "name": "table", "type": "table", "parts": [
        { "id": "col", "name": "Z", "type": "linear", "travel": { "min": -508.0, "max": 508.0 }, "safe_retract": 0.0, "parts": [
          { "id": "ram", "name": "Y", "type": "linear", "travel": { "min": -406.0, "max": 406.0 }, "parts": [
            { "id": "sld", "name": "X", "type": "linear", "travel": { "min": -660.0, "max": 660.0 } }
          ] }
        ] },
        { "id": "head", "name": "head", "type": "head",
          "spindle": { "min_speed": 100.0, "max_speed": 12000.0, "power": 11.0, "rigid_tapping": true },
          "tool_station": {
            "number_of_tools": 24,
            "max_tool_length": 250.0,
            "max_tool_diameter": 80.0,
            "spindle_mount": "CAT40",
            "default_holder": "CAT40 ER32",
            "coolants": [
              { "name": "Flood", "mCode": "M8" },
              { "name": "Mist", "mCode": "M7" }
            ]
          }
        }
      ] }
    ] } },
  "controller": { "coordinate_rotation": true, "synced_configuration": { "parts": {
    "X": { "coordinate": 0, "max_normal_speed": 28000.0, "max_rapid_speed": 28000.0 },
    "Y": { "coordinate": 1, "max_normal_speed": 28000.0, "max_rapid_speed": 28000.0 },
    "Z": { "coordinate": 2, "max_normal_speed": 28000.0, "max_rapid_speed": 28000.0 }
  } } }
}
