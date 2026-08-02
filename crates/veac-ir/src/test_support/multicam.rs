use crate::*;

use super::{range, sample_project, time};

pub(crate) fn multicam_project() -> ProjectEnvelope {
    let mut project = sample_project();
    let mut second = project.project.materials[0].clone();
    second.id = MaterialId::new("med_wide").unwrap();
    second.source = MaterialSource::File {
        uri: "media/wide.mp4".to_owned(),
    };
    let identity = MediaIdentity {
        algorithm: HashAlgorithm::Sha256,
        digest: "b".repeat(64),
    };
    second.identity = Some(identity.clone());
    second.probe.as_mut().unwrap().observed_identity = identity;
    project.project.materials.push(second);
    project.project.multicam_groups.push(MulticamGroup {
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
    });
    let clip = &mut project.project.sequences[0].tracks[0].clips[0];
    clip.source = ClipSource::Multicam {
        group_id: MulticamGroupId::new("mcg_interview").unwrap(),
        switches: vec![
            MulticamSwitch {
                angle_id: MulticamAngleId::new("ang_close").unwrap(),
                range: range(0, 300),
            },
            MulticamSwitch {
                angle_id: MulticamAngleId::new("ang_wide").unwrap(),
                range: range(300, 300),
            },
        ],
    };
    clip.source_mapping = None;
    project
}
