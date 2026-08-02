use veac_artifact::SourceClock;
use veac_plan::canonical::{Generator, MaterialKind, RationalTime, TimeRange};
use veac_plan::{ResolvedClip, ResolvedClipSource, ResolvedInputKind};

use super::error::{diagnostic, CodegenErrorKind};
use super::{time, CodegenErrors, EmitContext};

pub(super) fn video(
    context: &mut EmitContext<'_>,
    clip: &ResolvedClip,
) -> Result<String, CodegenErrors> {
    match &clip.source {
        ResolvedClipSource::Media {
            input_id,
            video_stream,
            ..
        } => {
            let mapping = source_mapping(clip)?;
            let Some(_selection) = video_stream.as_ref() else {
                return Err(unsupported(clip, "visual media has no video stream"));
            };
            let Some(input) = context
                .plan
                .inputs
                .iter()
                .find(|input| input.id == *input_id)
            else {
                return Err(missing_input(clip, input_id.to_string()));
            };
            if !matches!(
                input.kind,
                ResolvedInputKind::Media {
                    material_kind: MaterialKind::Video | MaterialKind::Image
                }
            ) {
                return Err(unsupported(clip, "input kind cannot produce visual frames"));
            }
            let image = matches!(
                input.kind,
                ResolvedInputKind::Media {
                    material_kind: MaterialKind::Image
                }
            );
            let Some(video) = input.video.as_ref() else {
                return Err(unsupported(clip, "resolved video facts are missing"));
            };
            super::video_source::media(context, clip, input_id, mapping, image, &video.info)
        }
        ResolvedClipSource::FreezeFrame {
            input_id,
            video_stream: _,
            source_time,
        } => {
            let (info, image) = context
                .plan
                .inputs
                .iter()
                .find(|input| input.id == *input_id)
                .and_then(|input| {
                    input.video.as_ref().map(|video| {
                        let image = matches!(
                            input.kind,
                            ResolvedInputKind::Media {
                                material_kind: MaterialKind::Image
                            }
                        );
                        (video.info.clone(), image)
                    })
                })
                .ok_or_else(|| unsupported(clip, "freeze-frame video facts are missing"))?;
            super::video_source::freeze(context, clip, input_id, *source_time, image, &info)
        }
        ResolvedClipSource::Generated { generator } => generated(context, clip, generator),
        ResolvedClipSource::Sequence { sequence_id } => nested(context, clip, sequence_id),
        ResolvedClipSource::Multicam { source } => {
            super::multicam_source::video(context, clip, source)
        }
        ResolvedClipSource::Text { .. } | ResolvedClipSource::Caption { .. } => {
            Err(unsupported(clip, "text is emitted by the text compositor"))
        }
    }
}

fn nested(
    context: &mut EmitContext<'_>,
    clip: &ResolvedClip,
    sequence_id: &veac_plan::canonical::SequenceId,
) -> Result<String, CodegenErrors> {
    let Some(sequence) = context
        .plan
        .sequences
        .iter()
        .find(|sequence| sequence.id == *sequence_id)
        .cloned()
    else {
        return Err(unsupported(
            clip,
            "nested sequence is missing from the plan",
        ));
    };
    let nested = super::sequence::build_nested(context, &sequence)?;
    let nested = context.graph.filter(
        &[&nested],
        format!(
            "settb=1/{},setpts=PTS-STARTPTS",
            sequence.duration.timescale
        ),
        "nestedtbv",
    );
    super::video_source::nested(
        context,
        clip,
        nested,
        source_mapping(clip)?,
        nested_clock(sequence.duration, clip)?,
    )
}

fn generated(
    context: &mut EmitContext<'_>,
    clip: &ResolvedClip,
    generator: &Generator,
) -> Result<String, CodegenErrors> {
    let color = match generator {
        Generator::Solid { color } => format!(
            "0x{:02X}{:02X}{:02X}{:02X}",
            color.red, color.green, color.blue, color.alpha
        ),
        Generator::Transparent => "black@0".to_owned(),
        Generator::Silence => return Err(unsupported(clip, "silence has no visual stream")),
        Generator::Gradient { .. } | Generator::Shape { .. } => {
            return Ok(super::generated::video(context, clip, generator))
        }
    };
    let canvas = context.canvas;
    Ok(context.graph.source(
        format!(
            "color=c={color}:s={}x{}:r={}/{}:d={},format=rgba",
            canvas.width,
            canvas.height,
            canvas.frame_rate.numerator,
            canvas.frame_rate.denominator,
            time::seconds(clip.record_range.duration)
        ),
        "generatedv",
    ))
}

pub(super) fn missing_input(clip: &ResolvedClip, id: String) -> CodegenErrors {
    CodegenErrors::one(diagnostic(
        CodegenErrorKind::MissingInput,
        "PLAN_INPUT_MISSING",
        Some(clip.id.to_string()),
        format!("clip references missing resolved input {id}"),
    ))
}

pub(super) fn unsupported(clip: &ResolvedClip, message: &str) -> CodegenErrors {
    CodegenErrors::one(diagnostic(
        CodegenErrorKind::UnsupportedSource,
        "SOURCE_UNSUPPORTED",
        Some(clip.id.to_string()),
        message,
    ))
}

pub(super) fn source_mapping(
    clip: &ResolvedClip,
) -> Result<&veac_plan::ResolvedSourceMapping, CodegenErrors> {
    clip.source_mapping
        .as_ref()
        .ok_or_else(|| unsupported(clip, "timed source has no source mapping"))
}

pub(super) fn nested_clock(
    duration: RationalTime,
    clip: &ResolvedClip,
) -> Result<SourceClock, CodegenErrors> {
    let zero = RationalTime::new(0, duration.timescale)
        .map_err(|_| unsupported(clip, "nested source clock is invalid"))?;
    let range = TimeRange::new(zero, duration)
        .map_err(|_| unsupported(clip, "nested source range is invalid"))?;
    SourceClock::bounded(range, zero)
        .map_err(|_| unsupported(clip, "nested source clock is invalid"))
}
