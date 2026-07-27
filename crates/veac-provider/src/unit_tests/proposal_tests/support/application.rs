use veac_ir::{ItemId, MaterialKind, OperationId, TimeRange, TrackId};

use crate::*;

use super::{audio_properties, output_material, time};

pub(crate) fn tts_context(artifact: &ProviderArtifact) -> ApplicationContext {
    ApplicationContext::TextToSpeech(Box::new(GeneratedAudioApplication {
        header: header("op_provider_tts"),
        output: audio_output(
            artifact,
            "med_provider_tts",
            "itm_provider_tts",
            TimeRange::new(time(0), time(10)).unwrap(),
        ),
    }))
}

pub(crate) fn dubbing_context(artifact: &ProviderArtifact) -> ApplicationContext {
    ApplicationContext::Dubbing(Box::new(DubbingApplication {
        header: header("op_provider_dubbing"),
        source_clip_id: ItemId::new("itm_audio").unwrap(),
        output: audio_output(
            artifact,
            "med_provider_dub",
            "itm_provider_dub",
            TimeRange::new(time(0), time(10)).unwrap(),
        ),
    }))
}

pub(crate) fn denoise_context(artifact: &ProviderArtifact) -> ApplicationContext {
    ApplicationContext::Denoise(Box::new(MediaReplacementApplication {
        header: header("op_provider_denoise"),
        material: output_material(
            artifact,
            "med_provider_denoise",
            MaterialKind::Audio,
            time(100),
        ),
        clip_id: ItemId::new("itm_audio").unwrap(),
        source_start: time(0),
    }))
}

pub(crate) fn removal_context(artifact: &ProviderArtifact) -> ApplicationContext {
    ApplicationContext::Removal(Box::new(MediaReplacementApplication {
        header: header("op_provider_removal"),
        material: output_material(
            artifact,
            "med_provider_removal",
            MaterialKind::Video,
            time(100),
        ),
        clip_id: ItemId::new("itm_video").unwrap(),
        source_start: time(0),
    }))
}

pub(crate) fn separation_context(stem: &SeparatedStem) -> ApplicationContext {
    ApplicationContext::VocalSeparation(Box::new(SeparationApplication {
        header: header("op_provider_separation"),
        source_clip_id: ItemId::new("itm_audio").unwrap(),
        bindings: vec![StemBinding {
            kind: stem.kind,
            label: stem.label.clone(),
            output: audio_output(
                &stem.audio,
                "med_provider_stem",
                "itm_provider_stem",
                TimeRange::new(time(0), time(100)).unwrap(),
            ),
        }],
    }))
}

fn audio_output(
    artifact: &ProviderArtifact,
    material_id: &str,
    clip_id: &str,
    record_range: TimeRange,
) -> AudioClipInsertion {
    AudioClipInsertion {
        material: output_material(
            artifact,
            material_id,
            MaterialKind::Audio,
            record_range.duration,
        ),
        sequence_id: veac_ir::SequenceId::new("seq_main").unwrap(),
        track_id: TrackId::new("trk_audio").unwrap(),
        clip_id: ItemId::new(clip_id).unwrap(),
        record_range,
        source_start: time(0),
        audio: audio_properties(),
        before_id: None,
        after_id: None,
    }
}

fn header(id: &str) -> ApplicationHeader {
    ApplicationHeader {
        project_revision: 7,
        operation_id: OperationId::new(id).unwrap(),
    }
}
