// Original CAMCraft metadata for its native Agie job writer.
// A single-file CPS surface cannot write a multi-file controller job.
description = "Agie native wire EDM: vertical punch, die opening or straight cutoff with quality review on Agie Control";
vendor = "Camcraft";
extension = "agie";
wireEdm = true;
programUnits = "mm";
declaredOptions = {
  separationLengthMm: {
    type: "number", title: "Separation cut length (mm)", group: "Agie technology",
    description: "Choose 1 mm for a closed contour. Choose zero for a straight open cutoff. Confirm the loaded quality before assigning the working steps in Agie Control.",
    min: 0, max: 1, default: 0
  }
};
function onOpen() {
  throw new Error("Agie native wire EDM requires the desktop or CLI One program post to a new job directory");
}
