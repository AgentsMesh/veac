use veac_plan::canonical::{CaptionNativeCue, CaptionSidecarFormat};

use super::{failure::Failure, Cue};

pub(super) fn validate(format: CaptionSidecarFormat, cue: &Cue<'_>) -> Result<(), Failure> {
    if !cue.semantics.spans.is_empty() {
        return Err(unsupported(
            cue,
            "CAPTION_NATIVE_SPANS_UNSUPPORTED",
            "caption-native markup spans cannot be preserved by this delivery path",
        ));
    }
    if !cue.semantics.words.is_empty() {
        return Err(unsupported(
            cue,
            "CAPTION_WORD_TIMING_UNSUPPORTED",
            "caption word timing requires a word-aware sidecar encoder",
        ));
    }
    if cue.semantics.style.is_some() {
        return Err(unsupported(
            cue,
            "CAPTION_NATIVE_STYLE_UNSUPPORTED",
            "caption-native style identity cannot be preserved by this delivery path",
        ));
    }
    let compatible = matches!(
        (&cue.semantics.native, format),
        (None, _)
            | (
                Some(CaptionNativeCue::Srt { .. }),
                CaptionSidecarFormat::Srt
            )
            | (
                Some(CaptionNativeCue::WebVtt { .. }),
                CaptionSidecarFormat::WebVtt
            )
            | (
                Some(CaptionNativeCue::Ass { .. }),
                CaptionSidecarFormat::Ass
            )
    );
    compatible.then_some(()).ok_or_else(|| {
        unsupported(
            cue,
            "CAPTION_NATIVE_FORMAT_MISMATCH",
            "caption-native settings cannot be converted to the selected sidecar format",
        )
    })
}

fn unsupported(cue: &Cue<'_>, code: &'static str, message: &str) -> Failure {
    Failure::unsupported(code, message).at(&cue.clip.id)
}
