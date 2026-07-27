use super::{ColorMatrix, ColorPrimaries, ColorRange, ColorTransfer};

use super::{output_keywords, OutputKeyword};

output_keywords!(ColorPrimaries,
    "bt709" => ColorPrimaries::Bt709,
    "bt470-m" => ColorPrimaries::Bt470M,
    "bt470-bg" => ColorPrimaries::Bt470Bg,
    "smpte170-m" => ColorPrimaries::Smpte170M,
    "smpte240-m" => ColorPrimaries::Smpte240M,
    "film" => ColorPrimaries::Film,
    "bt2020" => ColorPrimaries::Bt2020,
    "smpte428" => ColorPrimaries::Smpte428,
    "smpte431" => ColorPrimaries::Smpte431,
    "smpte432" => ColorPrimaries::Smpte432,
);
output_keywords!(ColorTransfer,
    "bt709" => ColorTransfer::Bt709,
    "gamma22" => ColorTransfer::Gamma22,
    "gamma28" => ColorTransfer::Gamma28,
    "smpte170-m" => ColorTransfer::Smpte170M,
    "smpte240-m" => ColorTransfer::Smpte240M,
    "linear" => ColorTransfer::Linear,
    "srgb" => ColorTransfer::Srgb,
    "bt2020-10" => ColorTransfer::Bt2020_10,
    "bt2020-12" => ColorTransfer::Bt2020_12,
    "smpte2084" => ColorTransfer::Smpte2084,
    "arib-std-b67" => ColorTransfer::AribStdB67,
);
output_keywords!(ColorMatrix,
    "rgb" => ColorMatrix::Rgb,
    "bt709" => ColorMatrix::Bt709,
    "fcc" => ColorMatrix::Fcc,
    "bt470-bg" => ColorMatrix::Bt470Bg,
    "smpte170-m" => ColorMatrix::Smpte170M,
    "smpte240-m" => ColorMatrix::Smpte240M,
    "ycgco" => ColorMatrix::Ycgco,
    "bt2020-ncl" => ColorMatrix::Bt2020Ncl,
);
output_keywords!(ColorRange,
    "limited" => ColorRange::Limited, "full" => ColorRange::Full,
);
