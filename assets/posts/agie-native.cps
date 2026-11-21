// Original CAMCraft metadata for its native Agie job writer.
// A single-file CPS surface cannot write a multi-file controller job.
description = "Agie native wire EDM: setup-only job for one vertical punch; review the technology on the control";
vendor = "Camcraft";
extension = "agie";
wireEdm = true;
programUnits = "mm";
declaredOptions = {
  separationLengthMm: {
    type: "number", title: "Separation cut length (mm)", group: "Agie technology",
    description: "Choose 1 mm for the native separation segment. Zero leaves it unspecified and blocks export. Review the effective work technology on the control before assigning its cutting steps.",
    min: 0, max: 1, default: 0
  }
};
function onOpen() {
  throw new Error("Agie native wire EDM requires the desktop or CLI One program post to a new job directory");
}
