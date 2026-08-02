use crate::source_edit::{
    ExpressionSite, SourceDeliveryArtifactKind, SourceDeliveryField, SourceNodeRef,
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
        .filter(|value| value.word(0) == Some("artifact"))
    {
        let Some(kind) = kind(entry.word(1)) else {
            continue;
        };
        let Some(id) = entry.id(2) else { continue };
        let target = SourceNodeRef::preset_delivery_artifact(&file.path, &preset.name, kind, id);
        if let Some(span) = entry.word_span(2) {
            index.register(&file.path, target.clone(), span)?;
        }
        fields(index, file, target, entry)?;
    }
    Ok(())
}

fn fields(
    index: &mut SourceIndex,
    file: &SurfaceFile,
    target: SourceNodeRef,
    entry: &Entry,
) -> Result<(), Diagnostic> {
    let Some(body) = &entry.block else {
        return Ok(());
    };
    for value in &body.entries {
        if value.word(0) == Some("target") {
            super::insert(
                index,
                file,
                target.clone(),
                ExpressionSite::PresetDeliveryField {
                    field: SourceDeliveryField::Target,
                },
                value,
                2,
            )?;
        }
        if value.word(0) == Some("encode") {
            encode(index, file, &target, value)?;
        }
    }
    Ok(())
}

fn encode(
    index: &mut SourceIndex,
    file: &SurfaceFile,
    target: &SourceNodeRef,
    entry: &Entry,
) -> Result<(), Diagnostic> {
    let Some(body) = &entry.block else {
        return Ok(());
    };
    for value in &body.entries {
        let field = match value.word(0) {
            Some("sample-format") => SourceDeliveryField::SampleFormat,
            Some("sample-rate") => SourceDeliveryField::SampleRate,
            Some("channel-layout") => SourceDeliveryField::ChannelLayout,
            _ => continue,
        };
        super::insert(
            index,
            file,
            target.clone(),
            ExpressionSite::PresetDeliveryField { field },
            value,
            1,
        )?;
    }
    Ok(())
}

fn kind(value: Option<&str>) -> Option<SourceDeliveryArtifactKind> {
    use SourceDeliveryArtifactKind as Kind;
    match value? {
        "video" => Some(Kind::Video),
        "image-sequence" => Some(Kind::ImageSequence),
        "caption-sidecar" => Some(Kind::CaptionSidecar),
        "audio-stem" => Some(Kind::AudioStem),
        "scope" => Some(Kind::Scope),
        "audio-file" => Some(Kind::AudioFile),
        "animated-image" => Some(Kind::AnimatedImage),
        "still-image" => Some(Kind::StillImage),
        "adaptive-package" => Some(Kind::AdaptivePackage),
        _ => None,
    }
}
