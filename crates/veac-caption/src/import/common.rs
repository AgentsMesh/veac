mod text;

use subtitler::model::{Subtitle, SubtitleFormat};
use veac_ir::{CaptionNativeCue, CaptionNativeId};

use crate::{
    time::range_from_millis, validate, CaptionCue, CaptionDocument, CaptionDocumentNative,
    CaptionEnvelope, CaptionError, CaptionFormat, ImportOptions, ImportResult, LossReport,
    WebVttHeader,
};

use super::{ids::IdFactory, native};

pub(super) fn import_srt(
    input: &str,
    options: &ImportOptions,
) -> Result<ImportResult, CaptionError> {
    let file = subtitler::srt::parse_content(input)
        .map_err(|error| CaptionError::parse(CaptionFormat::Srt, error))?;
    strict_count(
        input.matches("-->").count(),
        file.subtitles().len(),
        CaptionFormat::Srt,
    )?;
    build(
        file.subtitles(),
        Vec::new(),
        None,
        CaptionFormat::Srt,
        options,
        |_index, sub| {
            let native = sub.index.map(|value| value.to_string());
            let index = sub.index.map(|index| index as u64);
            Ok((native, index.map(|index| CaptionNativeCue::Srt { index })))
        },
    )
}

pub(super) fn import_vtt(
    input: &str,
    options: &ImportOptions,
) -> Result<ImportResult, CaptionError> {
    let signature = input
        .trim_start_matches('\u{feff}')
        .lines()
        .next()
        .map(str::trim)
        .unwrap_or_default();
    if signature != "WEBVTT" && !signature.starts_with("WEBVTT ") {
        return Err(CaptionError::parse(
            CaptionFormat::WebVtt,
            "missing WEBVTT signature",
        ));
    }
    let (raw_header, subtitles) = subtitler::vtt::parse_content_full(input)
        .map_err(|error| CaptionError::parse(CaptionFormat::WebVtt, error))?;
    strict_count(
        input.matches("-->").count(),
        subtitles.len(),
        CaptionFormat::WebVtt,
    )?;
    let ids = native::vtt_ids(input);
    let (header, unsupported_header) = webvtt_header(raw_header.as_deref());
    let mut result = build(
        &subtitles,
        Vec::new(),
        Some(CaptionDocumentNative::WebVtt { header }),
        CaptionFormat::WebVtt,
        options,
        |index, sub| {
            let native = ids.get(index).cloned().flatten();
            let settings = sub
                .settings
                .as_deref()
                .map(|value| {
                    crate::ir::semantics::webvtt::parse(value)
                        .map_err(|error| CaptionError::parse(CaptionFormat::WebVtt, error))
                })
                .transpose()?;
            Ok((
                native.clone(),
                Some(CaptionNativeCue::WebVtt {
                    identifier: native.map(CaptionNativeId),
                    settings,
                }),
            ))
        },
    )?;
    if unsupported_header {
        result.loss_report.document(
            "native.webvtt.header",
            "additional WebVTT header lines are outside the closed contract",
        );
    }
    if native::has_unsupported_vtt_markup(input) {
        result.loss_report.document(
            "text.spans",
            "WebVTT class, ruby, language, or timestamp markup is unsupported",
        );
    }
    validate(&CaptionEnvelope::new(result.document.clone()))?;
    Ok(result)
}

pub(super) fn build<F>(
    subtitles: &[Subtitle],
    styles: Vec<crate::CaptionStyle>,
    document_native: Option<CaptionDocumentNative>,
    format: CaptionFormat,
    options: &ImportOptions,
    mut native: F,
) -> Result<ImportResult, CaptionError>
where
    F: FnMut(usize, &Subtitle) -> Result<(Option<String>, Option<CaptionNativeCue>), CaptionError>,
{
    if subtitles.is_empty() {
        return Err(CaptionError::parse(format, "no valid cues found"));
    }
    let mut ids = IdFactory::new(&options.id_namespace);
    let mut cues = Vec::with_capacity(subtitles.len());
    for (index, subtitle) in subtitles.iter().enumerate() {
        let (native_id, cue_native) = native(index, subtitle)?;
        let id = ids.next(format, native_id.as_deref(), subtitle);
        let range = range_from_millis(subtitle.start, subtitle.end, options.timescale)?;
        let (mut caption_text, mut speaker) =
            text::from_parts(&subtitle.text, &subtitle.text_parts)?;
        if format == CaptionFormat::Ass {
            caption_text = text::from_ass(subtitle)?;
            speaker = None;
        }
        cues.push(CaptionCue {
            id,
            range,
            text: caption_text,
            speaker: subtitle.actor.clone().or(speaker),
            style: subtitle.style.clone(),
            native: cue_native,
            words: Vec::new(),
        });
    }
    let document = CaptionDocument {
        timescale: options.timescale,
        language: None,
        overlap_policy: options.overlap_policy,
        native: document_native,
        styles,
        cues,
    };
    validate(&CaptionEnvelope::new(document.clone()))?;
    Ok(ImportResult {
        document,
        loss_report: LossReport::default(),
    })
}

fn webvtt_header(raw: Option<&str>) -> (WebVttHeader, bool) {
    let mut lines = raw.unwrap_or("WEBVTT").lines();
    let signature = lines.next().unwrap_or("WEBVTT").trim();
    let description = signature
        .strip_prefix("WEBVTT")
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned);
    (WebVttHeader { description }, lines.next().is_some())
}

fn strict_count(markers: usize, parsed: usize, format: CaptionFormat) -> Result<(), CaptionError> {
    if markers == 0 || markers != parsed {
        Err(CaptionError::parse(
            format,
            "one or more cue records are malformed",
        ))
    } else {
        Ok(())
    }
}
