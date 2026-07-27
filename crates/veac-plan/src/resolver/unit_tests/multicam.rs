use super::support::*;
use crate::{canonical::*, plan_hash, resolve, ResolvedClipSource};

#[test]
fn multicam_resolution_owns_sync_angles_streams_and_switches() {
    let mut value = project();
    let mut wide = value.project.materials[0].clone();
    wide.id = MaterialId::new("med_wide").unwrap();
    wide.source = MaterialSource::File {
        uri: "media/wide.mp4".to_owned(),
    };
    let identity = identity('b');
    wide.identity = Some(identity.clone());
    wide.probe = Some(video_probe(identity));
    value.project.materials.push(wide);
    value.project.multicam_groups.push(group());
    value.project.render_configs[0]
        .video_deliverable_mut()
        .unwrap()
        .audio = Some(AudioOutput {
        codec: AudioCodec::Aac,
        sample_rate: 48_000,
        channels: 2,
    });
    let clip = &mut value.project.sequences[0].tracks[0].clips[0];
    clip.source = ClipSource::Multicam {
        group_id: MulticamGroupId::new("mcg_interview").unwrap(),
        switches: switches(),
    };
    clip.source_mapping = None;
    clip.audio = Some(audio_properties());

    let plan = resolve(&value, None).unwrap().remove(0);
    assert_eq!(plan.inputs.len(), 2);
    let ResolvedClipSource::Multicam { source } = &plan.sequences[0].tracks[0].clips[0].source
    else {
        panic!("expected owned multicam source")
    };
    assert_eq!(source.group_id.as_str(), "mcg_interview");
    assert_eq!(source.sync.basis, MulticamSyncBasis::Manual);
    assert_eq!(source.switches, switches());
    assert_eq!(source.angles.len(), 2);
    assert_eq!(source.angles[1].source_offset, time(60));
    assert!(source
        .angles
        .iter()
        .all(|angle| angle.audio_stream.is_some()));

    let original_hash = plan_hash(&plan).unwrap();
    let ClipSource::Multicam { switches, .. } =
        &mut value.project.sequences[0].tracks[0].clips[0].source
    else {
        unreachable!()
    };
    switches[0].angle_id = MulticamAngleId::new("ang_wide").unwrap();
    let changed = resolve(&value, None).unwrap().remove(0);
    assert_ne!(plan_hash(&changed).unwrap(), original_hash);
}

#[test]
fn audio_sync_keeps_angle_streams_when_the_clip_does_not_emit_audio() {
    let mut value = project();
    let mut wide = value.project.materials[0].clone();
    wide.id = MaterialId::new("med_wide").unwrap();
    wide.source = MaterialSource::File {
        uri: "media/wide.mp4".to_owned(),
    };
    let identity = identity('b');
    wide.identity = Some(identity.clone());
    wide.probe = Some(video_probe(identity));
    value.project.materials.push(wide);
    let mut multicam = group();
    multicam.sync.basis = MulticamSyncBasis::Audio;
    value.project.multicam_groups.push(multicam);
    let clip = &mut value.project.sequences[0].tracks[0].clips[0];
    clip.source = ClipSource::Multicam {
        group_id: MulticamGroupId::new("mcg_interview").unwrap(),
        switches: switches(),
    };
    clip.source_mapping = None;
    clip.audio = None;

    let plan = resolve(&value, None).unwrap().remove(0);
    let ResolvedClipSource::Multicam { source } = &plan.sequences[0].tracks[0].clips[0].source
    else {
        panic!("expected owned multicam source")
    };
    assert_eq!(source.sync.basis, MulticamSyncBasis::Audio);
    assert!(source
        .angles
        .iter()
        .all(|angle| angle.audio_stream.is_some()));
}

fn group() -> MulticamGroup {
    MulticamGroup {
        id: MulticamGroupId::new("mcg_interview").unwrap(),
        sync: MulticamSync {
            basis: MulticamSyncBasis::Manual,
            reference_angle_id: MulticamAngleId::new("ang_close").unwrap(),
        },
        angles: vec![
            MulticamAngle {
                id: MulticamAngleId::new("ang_close").unwrap(),
                material_id: MaterialId::new("med_video").unwrap(),
                source_offset: time(0),
            },
            MulticamAngle {
                id: MulticamAngleId::new("ang_wide").unwrap(),
                material_id: MaterialId::new("med_wide").unwrap(),
                source_offset: time(60),
            },
        ],
    }
}

fn switches() -> Vec<MulticamSwitch> {
    vec![
        MulticamSwitch {
            angle_id: MulticamAngleId::new("ang_close").unwrap(),
            range: range(0, 300),
        },
        MulticamSwitch {
            angle_id: MulticamAngleId::new("ang_wide").unwrap(),
            range: range(300, 300),
        },
    ]
}
