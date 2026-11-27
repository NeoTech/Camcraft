{
  "general": {
    "model": "Fehlmann Picomax 55 (iTNC 530)",
    "vendor": "Fehlmann",
    "kind": "mill",
    "description": "Three-axis CNC mode with a fixed W head. Set W before setting the preset. Sources: Fehlmann parameters 14108131.MP, operating manual 141-49-07-E and programming manual P07-35-06-E. Reference-frame travel limits are unstated. The preset table rows 0..5 come from the 2008 delivery CD (PRESET.PR); check the table at the control and raise controller.work_offset_max if the shop added rows."
  },
  "kinematics": { "default": {
    "units": { "length": "mm", "angle": "degrees" },
    "parts": [
      { "id": "x", "name": "X", "type": "linear", "parts": [
        { "id": "y", "name": "Y", "type": "linear", "parts": [
          { "id": "table", "name": "Table", "type": "table" }
        ] }
      ] },
      { "id": "z", "name": "Z", "type": "linear", "parts": [
        { "id": "head", "name": "Quill", "type": "head",
          "spindle": { "min_speed": 50, "max_speed": 9000 },
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
