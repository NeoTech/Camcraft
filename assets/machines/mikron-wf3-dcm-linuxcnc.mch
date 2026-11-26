{
  "general": {
    "model": "Mikron WF3 DCM (LinuxCNC)",
    "vendor": "Camcraft",
    "kind": "mill",
    "description": "Mikron WF3 DCM 3-axis mill retrofitted with LinuxCNC and Delta ASDA-A3 EtherCAT servos (XYZ 500x500x350 mm, X/Z 3000 mm/min, Y 2100 mm/min), ISO 40 (SK 40) manual tool-change spindle with encoder feedback (0-3000 rpm, rigid tapping), 800x450 mm table, flood coolant (M8). Provenance: paulsvc/linuxcnc-mill2-config (mill2.ini, README)."
  },
  "kinematics": {
    "default": {
      "units": { "length": "mm", "angle": "degrees" },
      "parts": [
        { "id": "base", "name": "table", "type": "table", "parts": [
          { "id": "col", "name": "Z", "type": "linear", "travel": { "min": -350.0, "max": 350.0 }, "parts": [
            { "id": "ram", "name": "Y", "type": "linear", "travel": { "min": -500.0, "max": 500.0 }, "parts": [
              { "id": "sld", "name": "X", "type": "linear", "travel": { "min": -500.0, "max": 500.0 } }
            ] }
          ] },
          { "id": "head", "name": "head", "type": "head",
            "spindle": { "max_speed": 3000.0, "rigid_tapping": true },
            "tool_station": {
              "coolants": [
                { "name": "Flood", "mCode": "M8" }
              ]
            }
          }
        ] }
      ]
    }
  },
  "controller": {
    "post": "LinuxCNC",
    "synced_configuration": {
      "parts": {
        "X": { "coordinate": 0, "max_normal_speed": 3000.0, "max_rapid_speed": 3000.0 },
        "Y": { "coordinate": 1, "max_normal_speed": 2100.0, "max_rapid_speed": 2100.0 },
        "Z": { "coordinate": 2, "max_normal_speed": 3000.0, "max_rapid_speed": 3000.0 }
      }
    }
  },
  "tooling": { "tool_change": "manual" }
}
