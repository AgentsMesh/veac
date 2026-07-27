use std::collections::BTreeMap;

use veac_artifact::ContentDigest;
use veac_ir::*;

use crate::test_support::{output_for, requests};
use crate::*;

use super::super::support::{project, time};

pub(super) fn fixture(
    basis: MulticamSyncBasis,
) -> (
    ProjectEnvelope,
    ProviderRequestEnvelope,
    ProviderResponseEnvelope,
    ApplicationContext,
) {
    let project = multicam_project(basis == MulticamSyncBasis::Audio);
    let payload = requests()
        .into_iter()
        .find(|value| value.capability() == Capability::MulticamSync)
        .unwrap();
    let mut request = ProviderRequestEnvelope::new(
        crate::test_support::negotiated(Capability::MulticamSync),
        payload,
    )
    .unwrap();
    let ProviderRequest::MulticamSync(parameters) = &mut request.request else {
        unreachable!()
    };
    parameters.basis = basis;
    for input in &mut parameters.angles {
        input.media.content = ContentDigest::sha256(b"Video");
        input.media.media_type = if basis == MulticamSyncBasis::Audio {
            MediaType::Audio
        } else {
            MediaType::Video
        };
        input.media.stream_index = Some(if basis == MulticamSyncBasis::Audio {
            1
        } else {
            0
        });
        input.timecode_start = (basis == MulticamSyncBasis::Timecode).then_some(time(0));
    }
    let mut output = output_for(&request);
    let ProviderOutput::MulticamSync(result) = &mut output else {
        unreachable!()
    };
    result.basis = basis;
    let response = ProviderResponseEnvelope::new(&request, output).unwrap();
    let context = ApplicationContext::MulticamSync(MulticamSyncApplication {
        header: ApplicationHeader {
            project_revision: 7,
            operation_id: OperationId::new("op_multicam_sync").unwrap(),
        },
        group_id: MulticamGroupId::new("mcg_main").unwrap(),
    });
    (project, request, response, context)
}

pub(super) fn multicam_project(with_audio: bool) -> ProjectEnvelope {
    let mut value = project();
    let mut second = value.project.materials[1].clone();
    second.id = MaterialId::new("med_source_video_b").unwrap();
    second.source = MaterialSource::File {
        uri: "fixtures/med_source_video_b.bin".into(),
    };
    value.project.materials.push(second);
    if with_audio {
        for material in &mut value.project.materials[1..] {
            attach_audio(material);
        }
    }
    value.project.multicam_groups.push(MulticamGroup {
        id: MulticamGroupId::new("mcg_main").unwrap(),
        sync: MulticamSync {
            basis: MulticamSyncBasis::Manual,
            reference_angle_id: MulticamAngleId::new("ang_a").unwrap(),
        },
        angles: vec![
            angle("ang_a", "med_source_video"),
            angle("ang_b", "med_source_video_b"),
        ],
    });
    veac_ir::validate(&value).unwrap();
    value
}

fn angle(id: &str, material: &str) -> MulticamAngle {
    MulticamAngle {
        id: MulticamAngleId::new(id).unwrap(),
        material_id: MaterialId::new(material).unwrap(),
        source_offset: time(0),
    }
}

fn attach_audio(material: &mut Material) {
    material.stream_intent.audio = StreamChoice::Auto;
    let probe = material.probe.as_mut().unwrap();
    probe.streams.push(ProbedStream {
        global_index: 1,
        type_index: 0,
        media_type: ProbedStreamType::Audio,
        codec: "pcm_s16le".into(),
        time_base: Some(Rational::new(1, 48_000).unwrap()),
        start_time: Some(time(0)),
        duration: Some(time(100)),
        disposition: StreamDisposition {
            default: true,
            attached_picture: false,
            timed_thumbnail: false,
        },
        video: None,
        audio: Some(AudioStreamInfo {
            sample_rate: 48_000,
            channels: 2,
            channel_layout: "stereo".into(),
        }),
    });
    probe.selected_audio_stream = Some(StreamSelection {
        global_index: 1,
        type_index: 0,
    });
    material.metadata = BTreeMap::new();
}
