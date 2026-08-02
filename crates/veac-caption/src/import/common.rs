mod text;

use std::collections::BTreeMap;

use subtitler::model::{Subtitle, SubtitleFormat};

use crate::{
    time::range_from_millis, validate, CaptionCue, CaptionDocument, CaptionEnvelope, CaptionError,
    CaptionFormat, ImportOptions, ImportResult, LossReport,
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
        CaptionFormat::Srt,
        options,
        |_index, sub| {
            let native = sub.index.map(|value| value.to_string());
            (
                native,
                BTreeMap::from_iter(
                    sub.index
                        .map(|value| ("srt.index".to_owned(), value.to_string())),
                ),
            )
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
    let (header, subtitles) = subtitler::vtt::parse_content_full(input)
        .map_err(|error| CaptionError::parse(CaptionFormat::WebVtt, error))?;
    strict_count(
        input.matches("-->").count(),
        subtitles.len(),
        CaptionFormat::WebVtt,
    )?;
    let ids = native::vtt_ids(input);
    let mut settings = BTreeMap::new();
    if let Some(header) = header {
        settings.insert("webvtt.header".to_owned(), header);
    }
    let mut result = build(
        &subtitles,
        Vec::new(),
        CaptionFormat::WebVtt,
        options,
        |index, sub| {
            let native = ids.get(index).cloned().flatten();
            let mut values = BTreeMap::new();
            if let Some(value) = &native {
                values.insert("webvtt.identifier".to_owned(), value.clone());
            }
            if let Some(value) = &sub.settings {
                values.insert("webvtt.settings".to_owned(), value.clone());
            }
            (native, values)
        },
    )?;
    result.document.settings = settings;
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
    format: CaptionFormat,
    options: &ImportOptions,
    mut native: F,
) -> Result<ImportResult, CaptionError>
where
    F: FnMut(usize, &Subtitle) -> (Option<String>, BTreeMap<String, String>),
{
    if subtitles.is_empty() {
        return Err(CaptionError::parse(format, "no valid cues found"));
    }
    let mut ids = IdFactory::new(&options.id_namespace);
    let mut cues = Vec::with_capacity(subtitles.len());
    for (index, subtitle) in subtitles.iter().enumerate() {
        let (native_id, settings) = native(index, subtitle);
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
            settings,
            words: Vec::new(),
        });
    }
    let document = CaptionDocument {
        timescale: options.timescale,
        language: None,
        overlap_policy: options.overlap_policy,
        settings: BTreeMap::new(),
        styles,
        cues,
    };
    validate(&CaptionEnvelope::new(document.clone()))?;
    Ok(ImportResult {
        document,
        loss_report: LossReport::default(),
    })
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
