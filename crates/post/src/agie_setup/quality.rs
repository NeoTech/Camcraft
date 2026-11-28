//! The work qualities that a zero-step piece can request, from the quality presets of our
//! own implementation and our offline Agie Control checks (#2954, #2984, #2966, #2986).

use super::QN2_OFFSETS_UM;

/// The one quality object of a piece. Its name identifies the requested
/// values and the wire, so that no generic name binds other resident data
/// (#2984). The control preselects the trims for the quality; the piece arms nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Quality {
    /// Fine Ra 0.30 µm, Te 1.50 µm, Tkm 1.00 µm, at every height in
    /// [`QN2_OFFSETS_UM`]. Every punch, die, group and cutoff uses it.
    #[default]
    Ccb030,
    /// Ra 0.35 µm, Te 1.50 µm, Tkm 1.00 µm with the speed option set, for one
    /// closed punch on a 20 mm plate only (#2966). See [`CCB035_OFFSETS_UM`].
    Ccb035,
    /// Ra 0.30 µm with Cobra Cut D 0.25 wire, for one closed punch on a 40 mm
    /// plate only (#2986). See [`CCD030_OFFSETS_UM`].
    Ccd030,
    /// Ra 0.70 µm with Cobra Cut D 0.25 wire, for one closed punch on a 40 mm
    /// plate only (#2986). See [`CCD070_OFFSETS_UM`].
    Ccd070,
}

/// The TECCUT technology that the control preselects for CCB035. The "Dis"
/// column of TECCUT is a match distance that changes with the request, so the
/// technology name is the stable key (#2966, control check of 25 Nov).
pub const CCB035_TECHNOLOGY: &str = "estbca25nsn060b035020";

/// Wire-centre offsets in µm of the main cut and the four trims on a
/// straight outside face for quality CCB035. With the speed option set, the
/// control showed the TECCUT header `TYPE:VS`, preselected
/// [`CCB035_TECHNOLOGY`] and plotted these five passes on a 20 mm Berco Cut
/// 0.25 punch in steel 0001 (#2966). No other height is measured.
pub const CCB035_OFFSETS_UM: [(u16, [u16; 5]); 1] = [(20, [234, 189, 154, 139, 129])];

/// Wire-centre offsets in µm of the main cut and the three trims on a
/// straight outside face for quality CCD030: one synthetic offline job on a
/// 40 mm Cobra Cut D 0.25 punch in steel 0001 (#2986). These are motion
/// values, not a surface measurement. No other height is measured.
pub const CCD030_OFFSETS_UM: [(u16, [u16; 4]); 1] = [(40, [250, 170, 145, 135])];

/// Wire-centre offsets in µm of the main cut and the two trims for quality
/// CCD070, from the second synthetic offline job of #2986 on the same punch.
pub const CCD070_OFFSETS_UM: [(u16, [u16; 3]); 1] = [(40, [245, 165, 140])];

impl Quality {
    /// The native quality name that the piece creates.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Ccb030 => "CCB030",
            Self::Ccb035 => "CCB035",
            Self::Ccd030 => "CCD030",
            Self::Ccd070 => "CCD070",
        }
    }

    /// The requested Ra in µm, as the control displays it.
    #[must_use]
    pub fn ra(self) -> &'static str {
        match self {
            Self::Ccb030 | Self::Ccd030 => "0.30",
            Self::Ccb035 => "0.35",
            Self::Ccd070 => "0.70",
        }
    }

    /// The native wire name that the piece and each of its works set.
    #[must_use]
    pub fn wire(self) -> &'static str {
        match self {
            Self::Ccb030 | Self::Ccb035 => "Berco Cut 0.25",
            Self::Ccd030 | Self::Ccd070 => "Cobra Cut D 0.25",
        }
    }

    /// The six `JE_CreateQuality` values: Ra, Te and Tkm in µm and three
    /// options. The first option is the speed flag ("Snelheid" on the control).
    pub(super) fn request(self) -> String {
        let speed = match self {
            Self::Ccb035 => "T",
            Self::Ccb030 | Self::Ccd030 | Self::Ccd070 => "F",
        };
        format!("{},1.50,1.00,{speed},F,F", self.ra())
    }

    /// The measured offsets of all passes for a stock thickness in µm, main
    /// cut first, by stock thickness in mm.
    fn table(self) -> Vec<(u16, &'static [u16])> {
        fn rows<const N: usize>(t: &'static [(u16, [u16; N])]) -> Vec<(u16, &'static [u16])> {
            t.iter().map(|(mm, o)| (*mm, &o[..])).collect()
        }
        match self {
            Self::Ccb030 => rows(&QN2_OFFSETS_UM),
            Self::Ccb035 => rows(&CCB035_OFFSETS_UM),
            Self::Ccd030 => rows(&CCD030_OFFSETS_UM),
            Self::Ccd070 => rows(&CCD070_OFFSETS_UM),
        }
    }

    /// The measured stock thicknesses in mm, in table order.
    #[must_use]
    pub fn thicknesses_mm(self) -> Vec<u16> {
        self.table().into_iter().map(|(mm, _)| mm).collect()
    }

    /// The measured offsets of all passes for a stock thickness in µm, main
    /// cut first, if it is qualified. CCB030 and CCD030 have 4 passes, CCB035
    /// has 5 and CCD070 has 3.
    #[must_use]
    pub fn offsets_um(self, height_um: i32) -> Option<&'static [u16]> {
        self.table()
            .into_iter()
            .find(|(mm, _)| i32::from(*mm) * 1000 == height_um)
            .map(|(_, o)| o)
    }

    /// The measured main-cut (rough) offset in µm for a stock thickness in µm.
    #[must_use]
    pub fn rough_um(self, height_um: i32) -> Option<u16> {
        self.offsets_um(height_um)?.first().copied()
    }

    /// The operator text for the requested options. The CCB030 text is that of #2984.
    pub(super) fn options_text(self) -> &'static str {
        match self {
            Self::Ccb030 | Self::Ccd030 | Self::Ccd070 => "three disabled options",
            Self::Ccb035 => "the speed option set, the other two options disabled",
        }
    }

    /// The operator text for the TECCUT step. The CCB030 text is that of #2984.
    pub(super) fn teccut_text(self) -> String {
        match self {
            Self::Ccb030 => "Assign the matching TECCUT row.".into(),
            Self::Ccb035 => format!(
                "TECCUT must show the header TYPE:VS and preselect technology {CCB035_TECHNOLOGY} \
                 with 5 passes. If not, stop. Assign it."
            ),
            Self::Ccd030 | Self::Ccd070 => format!(
                "TECCUT must preselect a Cobra Cut D 0.25 technology with {} passes. \
                 If not, stop. Assign it.",
                self.table().first().map_or(0, |(_, o)| o.len())
            ),
        }
    }
}
