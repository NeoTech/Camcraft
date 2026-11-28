{
  "general": {
    "model": "Fehlmann Picomax 55 (iTNC 530)",
    "vendor": "Fehlmann",
    "kind": "mill",
    "description": "Three-axis CNC mode with a fixed W head. Set W before setting the preset. Sources: the machine parameters, operating manual 141-49-07-E (section 7.1) and programming manual P07-35-06-E. Travel is the full stroke each way from the work origin: X 500, Y 255 and Z (quill) 147 mm. The spindle is the 9000 rpm option; the manual lists 7100 rpm for the standard spindle. The machine owner confirms rigid tapping. The manual names the SF32 tool holder; the SF32 mount is not stated because the holder library has no SF32 holders yet. The preset table rows 0..5 come from the 2008 delivery CD (PRESET.PR); check the table at the control and raise controller.work_offset_max if the shop added rows."
  },
  "kinematics": { "default": {
    "units": { "length": "mm", "angle": "degrees" },
    "parts": [
      { "id": "x", "name": "X", "type": "linear", "travel": { "min": -500.0, "max": 500.0 }, "parts": [
        { "id": "y", "name": "Y", "type": "linear", "travel": { "min": -255.0, "max": 255.0 }, "parts": [
          { "id": "table", "name": "Table", "type": "table" }
        ] }
      ] },
      { "id": "z", "name": "Z", "type": "linear", "travel": { "min": -147.0, "max": 147.0 }, "parts": [
        { "id": "head", "name": "Quill", "type": "head",
          "spindle": { "min_speed": 50, "max_speed": 9000, "rigid_tapping": true },
          "tool_station": { "coolants": [{ "name": "Flood", "mCode": "M8" }] }
        }
      ] }
    ]
  } },
  "controller": {
    "post": "Heidenhain Klartext",
    "identity": { "model": "iTNC 530", "nc_software": "340490-03" },
    "work_offset_max": 5,
    "synced_configuration": { "parts": {
      "X": { "coordinate": 0, "max_rapid_speed": 11000 },
      "Y": { "coordinate": 1, "max_rapid_speed": 11000 },
      "Z": { "coordinate": 2, "max_rapid_speed": 6000 }
    } }
  },
  "tooling": { "tool_change": "manual-retract-top", "max_tool_number": 254 }
}
