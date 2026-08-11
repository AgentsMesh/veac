use super::{Effect, EffectKind, PluginEffectDigest};
use crate::{Animatable, Color, REFERENCE_MONOCHROME_DIGEST};

impl Effect {
    pub fn neutral(kind: EffectKind) -> Self {
        let zero = Animatable::constant(0.0);
        let one = Animatable::constant(1.0);
        match kind {
            EffectKind::VideoColorAdjust => Self::VideoColorAdjust {
                brightness: zero,
                contrast: one.clone(),
                saturation: one,
            },
            EffectKind::VideoBlur => Self::VideoBlur { radius: zero },
            EffectKind::VideoDirectionalBlur => Self::VideoDirectionalBlur {
                angle_degrees: zero.clone(),
                radius: zero,
            },
            EffectKind::VideoSharpen => Self::VideoSharpen { amount: zero },
            EffectKind::VideoVignette => Self::VideoVignette { amount: zero },
            EffectKind::VideoGrain => Self::VideoGrain { amount: zero },
            EffectKind::VideoChromaKey => Self::VideoChromaKey {
                color: green(),
                similarity: Animatable::constant(0.1),
                blend: zero,
            },
            EffectKind::VideoLumaKey => Self::VideoLumaKey {
                threshold: zero.clone(),
                tolerance: Animatable::constant(0.01),
                softness: zero,
                invert: false,
            },
            EffectKind::VideoChromaSpill => Self::VideoChromaSpill {
                color: green(),
                amount: Animatable::constant(0.5),
                range: zero,
            },
            EffectKind::VideoStabilize => Self::VideoStabilize { enabled: true },
            EffectKind::AudioNormalize => Self::AudioNormalize { target_lufs: -16.0 },
            EffectKind::VideoPluginReferenceMonochromeV1 => {
                Self::VideoPluginReferenceMonochromeV1 {
                    descriptor_digest: PluginEffectDigest::new(REFERENCE_MONOCHROME_DIGEST)
                        .expect("reference plugin digest is canonical"),
                    amount: zero,
                }
            }
        }
    }
}

const fn green() -> Color {
    Color {
        red: 0,
        green: 255,
        blue: 0,
        alpha: 255,
    }
}
