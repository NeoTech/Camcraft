// Original CAMCraft metadata for its native Agie job writer.
// A single-file CPS surface cannot write a multi-file controller job.
description = "Native wire EDM job files: setup-only job for one vertical punch; review the technology on the control";
vendor = "Camcraft";
extension = "agie";
wireEdm = true;
programUnits = "mm";
declaredOptions = {
  separationLengthMm: {
    type: "number", title: "Separation cut length (mm)", group: "Wire EDM technology",
    description: "Choose 1 mm for the native separation segment. Zero leaves it unspecified and blocks export. Review the effective work technology on the control before assigning its cutting steps.",
    min: 0, max: 1, default: 0
  }
};
function onOpen() {
  throw new Error("Native wire EDM job files require the desktop or CLI One program post to a new job directory");
}
