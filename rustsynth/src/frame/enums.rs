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
            1 => ColorPrimaries::BT709,
            2 => ColorPrimaries::UNSPECIFIED,
            4 => ColorPrimaries::Bt470M,
            5 => ColorPrimaries::Bt470Bg,
            6 => ColorPrimaries::St170M,
            7 => ColorPrimaries::St240M,
            8 => ColorPrimaries::FILM,
            9 => ColorPrimaries::BT2020,
            10 => ColorPrimaries::ST428,
            11 => ColorPrimaries::ST431_2,
            12 => ColorPrimaries::ST432_1,
            22 => ColorPrimaries::Ebu3213E,
            _ => ColorPrimaries::UNSPECIFIED, // fallback
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
            0 => MatrixCoefficients::RGB,
            1 => MatrixCoefficients::BT709,
            2 => MatrixCoefficients::UNSPECIFIED,
            4 => MatrixCoefficients::FCC,
            5 => MatrixCoefficients::Bt470Bg,
            6 => MatrixCoefficients::St170M,
            7 => MatrixCoefficients::St240M,
            8 => MatrixCoefficients::YCGCO,
            9 => MatrixCoefficients::Bt2020Ncl,
            10 => MatrixCoefficients::Bt2020Cl,
            12 => MatrixCoefficients::ChromaticityDerivedNcl,
            13 => MatrixCoefficients::ChromaticityDerivedCl,
            14 => MatrixCoefficients::ICTCP,
            _ => MatrixCoefficients::UNSPECIFIED, // fallback
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
            1 => TransferCharacteristics::BT709,
            2 => TransferCharacteristics::UNSPECIFIED,
            4 => TransferCharacteristics::Bt470M,
            5 => TransferCharacteristics::Bt470Bg,
            6 => TransferCharacteristics::BT601,
            7 => TransferCharacteristics::St240M,
            8 => TransferCharacteristics::LINEAR,
            9 => TransferCharacteristics::Log100,
            10 => TransferCharacteristics::Log316,
            11 => TransferCharacteristics::Iec61966_2_4,
            13 => TransferCharacteristics::Iec61966_2_1,
            14 => TransferCharacteristics::BT2020_10,
            15 => TransferCharacteristics::BT2020_12,
            16 => TransferCharacteristics::ST2084,
            17 => TransferCharacteristics::ST428,
            18 => TransferCharacteristics::AribB67,
            _ => TransferCharacteristics::UNSPECIFIED, // fallback
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
