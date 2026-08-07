use super::super::{ColorMatrix, ColorPrimaries, ColorRange, ColorTransfer};
use veac_ir::{
    ColorMatrix as CanonicalMatrix, ColorPrimaries as CanonicalPrimaries,
    ColorRange as CanonicalRange, ColorTransfer as CanonicalTransfer,
};

macro_rules! color_syntax {
    ($authored:ident, $canonical:ident, $($variant:ident => $token:literal),+ $(,)?) => {
        crate::impl_local_syntax_tokens!(
            $authored,
            $($authored::$variant => $token),+
        );

        impl From<$authored> for $canonical {
            fn from(value: $authored) -> Self {
                match value {
                    $($authored::$variant => Self::$variant),+
                }
            }
        }

        impl From<$canonical> for $authored {
            fn from(value: $canonical) -> Self {
                match value {
                    $($canonical::$variant => Self::$variant),+
                }
            }
        }
    };
}

color_syntax!(ColorPrimaries, CanonicalPrimaries,
    Bt709 => "bt709", Bt470M => "bt470-m", Bt470Bg => "bt470-bg",
    Smpte170M => "smpte170-m", Smpte240M => "smpte240-m", Film => "film",
    Bt2020 => "bt2020", Smpte428 => "smpte428", Smpte431 => "smpte431",
    Smpte432 => "smpte432",
);
color_syntax!(ColorTransfer, CanonicalTransfer,
    Bt709 => "bt709", Gamma22 => "gamma22", Gamma28 => "gamma28",
    Smpte170M => "smpte170-m", Smpte240M => "smpte240-m", Linear => "linear",
    Srgb => "srgb", Bt2020_10 => "bt2020-10", Bt2020_12 => "bt2020-12",
    Smpte2084 => "smpte2084", AribStdB67 => "arib-std-b67",
);
color_syntax!(ColorMatrix, CanonicalMatrix,
    Rgb => "rgb", Bt709 => "bt709", Fcc => "fcc", Bt470Bg => "bt470-bg",
    Smpte170M => "smpte170-m", Smpte240M => "smpte240-m", Ycgco => "ycgco",
    Bt2020Ncl => "bt2020-ncl",
);
color_syntax!(ColorRange, CanonicalRange,
    Limited => "limited", Full => "full",
);

#[cfg(test)]
mod tests;
