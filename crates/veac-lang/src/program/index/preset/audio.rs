use crate::source_edit::{
    ExpressionSite, SourceAudioEqBandField, SourceAudioProcessorField, SourceAudioProcessorKind,
    SourceNodeRef,
};

use super::super::super::diagnostic::Diagnostic;
use super::super::super::model::{PresetDecl, SurfaceFile};
use super::super::syntax::{Block, Entry};
use super::super::SourceIndex;

pub(super) fn index(
    index: &mut SourceIndex,
    file: &SurfaceFile,
    preset: &PresetDecl,
    body: &Block,
) -> Result<(), Diagnostic> {
    for entry in body
        .entries
        .iter()
        .filter(|value| value.word(0) == Some("processor"))
    {
        let Some(kind) = kind(entry.word(1)) else {
            continue;
        };
        let Some(processor) = entry.id(2) else {
            continue;
        };
        let target = SourceNodeRef::preset_audio_processor(&file.path, &preset.name, processor);
        if let Some(span) = entry.word_span(2) {
            index.register(&file.path, target.clone(), span)?;
        }
        fields(index, file, target, kind, entry)?;
        if kind == SourceAudioProcessorKind::Eq {
            eq_bands(index, file, preset, processor, entry)?;
        }
    }
    Ok(())
}

fn eq_bands(
    index: &mut SourceIndex,
    file: &SurfaceFile,
    preset: &PresetDecl,
    processor: &str,
    entry: &Entry,
) -> Result<(), Diagnostic> {
    let Some(body) = &entry.block else {
        return Ok(());
    };
    for band in body
        .entries
        .iter()
        .filter(|value| value.word(0) == Some("band"))
    {
        let Some(id) = band.id(1) else { continue };
        let target = SourceNodeRef::preset_audio_eq_band(&file.path, &preset.name, processor, id);
        if let Some(span) = band.word_span(1) {
            index.register(&file.path, target.clone(), span)?;
        }
        eq_band_fields(index, file, target, band)?;
    }
    Ok(())
}

fn eq_band_fields(
    index: &mut SourceIndex,
    file: &SurfaceFile,
    target: SourceNodeRef,
    entry: &Entry,
) -> Result<(), Diagnostic> {
    let Some(body) = &entry.block else {
        return Ok(());
    };
    for value in &body.entries {
        let field = match value.word(0) {
            Some("frequency") => SourceAudioEqBandField::Frequency,
            Some("gain") => SourceAudioEqBandField::Gain,
            Some("q") => SourceAudioEqBandField::Q,
            _ => continue,
        };
        super::insert(
            index,
            file,
            target.clone(),
            ExpressionSite::PresetAudioEqBandField { field },
            value,
            1,
        )?;
    }
    Ok(())
}

fn fields(
    index: &mut SourceIndex,
    file: &SurfaceFile,
    target: SourceNodeRef,
    kind: SourceAudioProcessorKind,
    entry: &Entry,
) -> Result<(), Diagnostic> {
    let Some(body) = &entry.block else {
        return Ok(());
    };
    for value in &body.entries {
        let Some(field) = field(kind, value.word(0)) else {
            continue;
        };
        super::insert(
            index,
            file,
            target.clone(),
            ExpressionSite::PresetAudioProcessorField {
                processor_kind: kind,
                field,
            },
            value,
            1,
        )?;
    }
    Ok(())
}

fn kind(value: Option<&str>) -> Option<SourceAudioProcessorKind> {
    use SourceAudioProcessorKind as Kind;
    match value? {
        "eq" => Some(Kind::Eq),
        "high-pass" => Some(Kind::HighPass),
        "low-pass" => Some(Kind::LowPass),
        "compressor" => Some(Kind::Compressor),
        "limiter" => Some(Kind::Limiter),
        "gate" => Some(Kind::Gate),
        "loudness" => Some(Kind::Loudness),
        _ => None,
    }
}

fn field(
    processor: SourceAudioProcessorKind,
    value: Option<&str>,
) -> Option<SourceAudioProcessorField> {
    use SourceAudioProcessorField as Field;
    use SourceAudioProcessorKind as Kind;
    Some(match (processor, value?) {
        (Kind::HighPass | Kind::LowPass, "frequency") => Field::Frequency,
        (Kind::HighPass | Kind::LowPass, "q") => Field::Q,
        (Kind::HighPass | Kind::LowPass, "poles") => Field::Poles,
        (Kind::Compressor | Kind::Gate, "threshold") => Field::Threshold,
        (Kind::Compressor | Kind::Gate, "ratio") => Field::Ratio,
        (Kind::Compressor | Kind::Limiter | Kind::Gate, "attack") => Field::Attack,
        (Kind::Compressor | Kind::Limiter | Kind::Gate, "release") => Field::Release,
        (Kind::Compressor, "knee") => Field::Knee,
        (Kind::Compressor, "makeup-gain") => Field::MakeupGain,
        (Kind::Compressor, "mix") => Field::Mix,
        (Kind::Limiter, "ceiling") => Field::Ceiling,
        (Kind::Gate | Kind::Loudness, "range") => Field::Range,
        (Kind::Loudness, "integrated") => Field::Integrated,
        (Kind::Loudness, "true-peak") => Field::TruePeak,
        _ => return None,
    })
}
