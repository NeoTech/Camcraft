//! Split from `types.rs` by M5.0.1; structure only.

/// What a height is measured from. Every source but [`HeightReference::Fixed`]
/// is a world truth re-read at resolve time - the stock and model bounding
/// boxes and the work-offset slot placements - so a height that follows the
/// moved setup is expressed, never stale. Rotations are not part of a Z
/// reference: only the source's height feeds an offset (a full WCS frame
/// transform is out of 2D scope, like 3D itself).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HeightReference {
    /// The offset is the Z itself, in world (model) coordinates. What
    /// pre-reference files and hand-typed absolute values carry.
    Fixed,
    TopOfStock,
    BottomOfStock,
    TopOfModel,
    BottomOfModel,
    /// The Z=0 plane of the operation's SETUP work offset (#329): the
    /// slot [`crate::setup::wcs_slot`] names for the op's setup, never a
    /// slot of the height's own - program zero is the setup's, so every
    /// operation of one setup measures from the same offset. An
    /// existing-but-unplaced slot reads as the world origin, exactly as
    /// the rest of the world treats an entity nobody has moved.
    WorkOffset,
    /// The highest Z of the operation's OWN source geometry (M11.3 A):
    /// its picked features, its named entity's bounds, or the setup's
    /// stock box - [`crate::operation::selection_z_range`] states the
    /// rule per source. Appended after every older reference so no
    /// existing kind index, stamp tag or file word moves.
    TopOfSelection,
    /// The lowest Z of the same geometry (a picked flat face: its Z).
    BottomOfSelection,
}

impl HeightReference {
    /// Panel combo entries, in this order; the panel shows the setup's
    /// offset (read-only) under a WorkOffset reference.
    /// "Selection" is ONE entry for both selection ends: the height it is
    /// chosen for decides the end ([`Self::selection_for`]).
    pub const KIND_LABELS: [&'static str; 7] = [
        "Fixed value",
        "Top of stock",
        "Bottom of stock",
        "Top of model",
        "Bottom of model",
        "Work offset",
        "Selection",
    ];

    /// Which entry of [`Self::KIND_LABELS`] this reference's kind is.
    #[must_use]
    pub fn kind_index(&self) -> usize {
        match self {
            HeightReference::Fixed => 0,
            HeightReference::TopOfStock => 1,
            HeightReference::BottomOfStock => 2,
            HeightReference::TopOfModel => 3,
            HeightReference::BottomOfModel => 4,
            HeightReference::WorkOffset => 5,
            HeightReference::TopOfSelection | HeightReference::BottomOfSelection => 6,
        }
    }

    /// The "Selection" entry spoken for one height slot (M11.3 A): a
    /// bottom (floor / cut / parting plane) measures from the
    /// selection's lowest Z; clearance, retract and top from its highest.
    #[must_use]
    pub fn selection_for(bottom: bool) -> Self {
        if bottom {
            HeightReference::BottomOfSelection
        } else {
            HeightReference::TopOfSelection
        }
    }

    /// Whether this reference reads the operation's own selection.
    #[must_use]
    pub fn is_selection(&self) -> bool {
        matches!(self, HeightReference::TopOfSelection | HeightReference::BottomOfSelection)
    }

}

impl core::fmt::Display for HeightReference {
    /// Chip text for a plane label: short. A work offset is the setup's
    /// (#329), so the chip names the reference, not a slot.
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            HeightReference::Fixed => f.write_str("fixed"),
            HeightReference::TopOfStock => f.write_str("stock top"),
            HeightReference::BottomOfStock => f.write_str("stock bottom"),
            HeightReference::TopOfModel => f.write_str("model top"),
            HeightReference::BottomOfModel => f.write_str("model bottom"),
            HeightReference::WorkOffset => f.write_str("work offset"),
            HeightReference::TopOfSelection => f.write_str("selection top"),
            HeightReference::BottomOfSelection => f.write_str("selection bottom"),
        }
    }
}

/// One height: From-reference plus offset in mm (HSMWorks-style "From:
/// model top, Offset: 0.5"). The stored truth is the expression, never a
/// resolved Z - so a height cannot disagree with the setup it references.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Height {
    pub reference: HeightReference,
    pub offset: f64,
}

impl Height {
    /// A height that stands on its own: the value is the world Z.
    #[must_use]
    pub fn fixed(z: f64) -> Self {
        Height { reference: HeightReference::Fixed, offset: z }
    }

    #[must_use]
    pub fn of(reference: HeightReference, offset: f64) -> Self {
        Height { reference, offset }
    }

    /// Structural sanity, no world: a finite offset. The slot a
    /// WorkOffset reference reads is the setup's, validated where the
    /// setup picks it (#329). Ordering lives on resolved values
    /// ([`crate::operation::depth_sane`]).
    #[must_use]
    pub fn is_sane(&self) -> bool {
        self.offset.is_finite()
    }

    #[must_use]
    pub fn is_fixed(&self) -> bool {
        matches!(self.reference, HeightReference::Fixed)
    }
}

/// What a rotary height is measured from (#2774). A rotary tool must keep
/// its distance from the fitted axis all the way round, so its heights are
/// RADII about that axis, never world-Z planes. The two geometry sources
/// are world truths re-read at resolve time - so a radius that follows the
/// moved stock or a re-imported model is expressed, never stale - exactly
/// the law [`Height`] keeps for its Z references.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RadialSource {
    /// The setup stock's largest radius about the fitted axis.
    StockOd,
    /// The operation's own model's largest radius about the fitted axis.
    ModelMaxRadius,
    /// The offset IS the radius, in mm from the axis.
    Absolute,
}

impl RadialSource {
    /// Panel combo entries, in this order.
    pub const KIND_LABELS: [&'static str; 3] = ["stock OD", "model max radius", "absolute radius"];

    /// Which entry of [`Self::KIND_LABELS`] this source is.
    #[must_use]
    pub fn kind_index(&self) -> usize {
        match self {
            RadialSource::StockOd => 0,
            RadialSource::ModelMaxRadius => 1,
            RadialSource::Absolute => 2,
        }
    }

    /// The source that a combo entry selects.
    #[must_use]
    pub fn from_index(i: usize) -> Self {
        match i {
            0 => RadialSource::StockOd,
            1 => RadialSource::ModelMaxRadius,
            _ => RadialSource::Absolute,
        }
    }
}

impl core::fmt::Display for RadialSource {
    /// Chip text for a radius label: short.
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            RadialSource::StockOd => f.write_str("stock OD"),
            RadialSource::ModelMaxRadius => f.write_str("model max radius"),
            RadialSource::Absolute => f.write_str("absolute"),
        }
    }
}

/// One rotary height (#2774): a From-source plus offset in mm, read as a
/// RADIUS about the fitted rotary axis. The stored truth is the expression,
/// never a resolved radius - so a radius cannot disagree with the bar it
/// references. For [`RadialSource::Absolute`] the offset IS the radius; for
/// the two relative sources the radius is the source's own radius plus the
/// offset (a negative offset is legal - it names a radius inside the skin,
/// which the gate refuses by name when it is not safe).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RadialHeight {
    pub source: RadialSource,
    pub offset: f64,
}

impl RadialHeight {
    /// A radius that stands on its own: the value is the mm from the axis.
    #[must_use]
    pub fn absolute(r: f64) -> Self {
        RadialHeight { source: RadialSource::Absolute, offset: r }
    }

    #[must_use]
    pub fn of(source: RadialSource, offset: f64) -> Self {
        RadialHeight { source, offset }
    }

    /// Structural sanity, no world: a finite offset, and an ABSOLUTE
    /// radius that is a distance above the axis (a negative absolute
    /// radius is no radius at all - the stored expression is the radius
    /// there). Ordering lives on resolved radii
    /// ([`crate::operation::radial_gate`]).
    #[must_use]
    pub fn is_sane(&self) -> bool {
        self.offset.is_finite() && (!self.is_absolute() || self.offset >= 0.0)
    }

    #[must_use]
    pub fn is_absolute(&self) -> bool {
        matches!(self.source, RadialSource::Absolute)
    }
}
