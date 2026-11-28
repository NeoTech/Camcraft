{
  "general": {
    "model": "G-EDM WIRE EDM (TEMPLATE)",
    "vendor": "Camcraft",
    "kind": "wire_edm",
    "description": "Template for a G-EDM wire EDM build on a G-RBLv3 controller, from the public G-EDM pages (gedm.org): 2-axis X/Y wire mode, spark power set at the controller, no automatic wire threading. The template states no travels, no rapids, no guide heights, no taper and no cutting conditions, because each build is different. Before a Wire Contour: open the Machine editor, enter the X/Y travels and the guide heights of your build, and add one condition (EDM) for each wire and setting with the spark gap that you measured. Its post is G-EDM Wire EDM; the post output is not verified on a G-EDM controller."
  },
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
  "controller": { "post": "G-EDM Wire EDM" },
  "edm": { "awt": false }
}
