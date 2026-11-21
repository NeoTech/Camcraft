// Original CAMCraft metadata for its native Agie job writer.
// A single-file CPS surface cannot write a multi-file controller job.
description = "Agie native wire EDM: one vertical punch, die opening or straight cutoff, controller compensation and QN2 technology";
vendor = "Camcraft";
extension = "agie";
wireEdm = true;
programUnits = "mm";
declaredOptions = {
  separationLengthMm: {
    type: "number", title: "Separation cut length (mm)", group: "Agie technology",
    description: "Choose 1 mm for a closed contour. Choose zero for a straight open cutoff. The controller generates the separation and trim sequence.",
    min: 0, max: 1, default: 0
  }
};
function onOpen() {
  throw new Error("Agie native wire EDM requires the desktop or CLI One program post to a new job directory");
}
