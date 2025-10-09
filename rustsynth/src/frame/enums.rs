#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorPrimaries {
    BT709 = 1,
    UNSPECIFIED = 2,
    Bt470M = 4,
    Bt470Bg = 5,
    St170M = 6,
    St240M = 7,
    FILM = 8,
    BT2020 = 9,
    ST428 = 10,
    ST431_2 = 11,
    ST432_1 = 12,
    Ebu3213E = 22,
}

impl From<i64> for ColorPrimaries {
    fn from(value: i64) -> Self {
        match value {
            1 => Self::BT709,
            2 => Self::UNSPECIFIED,
            4 => Self::Bt470M,
            5 => Self::Bt470Bg,
            6 => Self::St170M,
            7 => Self::St240M,
            8 => Self::FILM,
            9 => Self::BT2020,
            10 => Self::ST428,
            11 => Self::ST431_2,
            12 => Self::ST432_1,
            22 => Self::Ebu3213E,
            _ => Self::UNSPECIFIED, // fallback
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatrixCoefficients {
    RGB = 0,
    BT709 = 1,
    UNSPECIFIED = 2,
    FCC = 4,
    Bt470Bg = 5,
    St170M = 6,
    St240M = 7,
    YCGCO = 8,
    Bt2020Ncl = 9,
    Bt2020Cl = 10,
    ChromaticityDerivedNcl = 12,
    ChromaticityDerivedCl = 13,
    ICTCP = 14,
}

impl From<i64> for MatrixCoefficients {
    fn from(value: i64) -> Self {
        match value {
            0 => Self::RGB,
            1 => Self::BT709,
            2 => Self::UNSPECIFIED,
            4 => Self::FCC,
            5 => Self::Bt470Bg,
            6 => Self::St170M,
            7 => Self::St240M,
            8 => Self::YCGCO,
            9 => Self::Bt2020Ncl,
            10 => Self::Bt2020Cl,
            12 => Self::ChromaticityDerivedNcl,
            13 => Self::ChromaticityDerivedCl,
            14 => Self::ICTCP,
            _ => Self::UNSPECIFIED, // fallback
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferCharacteristics {
    BT709 = 1,
    UNSPECIFIED = 2,
    Bt470M = 4,
    Bt470Bg = 5,
    BT601 = 6,
    St240M = 7,
    LINEAR = 8,
    Log100 = 9,
    Log316 = 10,
    Iec61966_2_4 = 11,
    Iec61966_2_1 = 13,
    BT2020_10 = 14,
    BT2020_12 = 15,
    ST2084 = 16,
    ST428 = 17,
    AribB67 = 18,
}

impl From<i64> for TransferCharacteristics {
    fn from(value: i64) -> Self {
        match value {
            1 => Self::BT709,
            2 => Self::UNSPECIFIED,
            4 => Self::Bt470M,
            5 => Self::Bt470Bg,
            6 => Self::BT601,
            7 => Self::St240M,
            8 => Self::LINEAR,
            9 => Self::Log100,
            10 => Self::Log316,
            11 => Self::Iec61966_2_4,
            13 => Self::Iec61966_2_1,
            14 => Self::BT2020_10,
            15 => Self::BT2020_12,
            16 => Self::ST2084,
            17 => Self::ST428,
            18 => Self::AribB67,
            _ => Self::UNSPECIFIED, // fallback
        }
    }
}

/// Chroma sample position in YUV formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChromaLocation {
    Left = 0,
    Center = 1,
    TopLeft = 2,
    Top = 3,
    BottomLeft = 4,
    Bottom = 5,
}

/// Full or limited range (PC/TV range)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorRange {
    Full = 0,
    Limited = 1,
}

/// If the frame is composed of two independent fields (interlaced)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldBased {
    Progressive = 0,
    BottomFieldFirst = 1,
    TopFieldFirst = 2,
}

/// Which field was used to generate this frame
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    Bottom = 0,
    Top = 1,
}
