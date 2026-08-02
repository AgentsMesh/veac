mod support;

use support::*;
use veac_plan::canonical::*;
use veac_plan::{plan_hash, resolve_one, ResolvedClipSource};

#[test]
fn multicam_plan_owns_every_angle_sync_offset_and_switch_segment() {
    let mut project = project();
    let mut wide = project.project.materials[0].clone();
    wide.id = MaterialId::new("med_wide").unwrap();
    wide.source = MaterialSource::File {
        uri: "media/wide.mp4".to_owned(),
    };
    let identity = identity('b');
    wide.identity = Some(identity.clone());
    wide.probe = Some(video_probe(identity));
    project.project.materials.push(wide);
    project.project.multicam_groups.push(group());
    let clip = &mut project.project.sequences[0].tracks[0].clips[0];
    clip.source = ClipSource::Multicam {
        group_id: MulticamGroupId::new("mcg_interview").unwrap(),
        switches: switches(),
    };
    clip.source_mapping = None;
    let output_id = project.project.render_configs[0].id.clone();
    let plan = resolve_one(&project, &output_id).unwrap();
    assert_eq!(plan.inputs.len(), 2);
    let ResolvedClipSource::Multicam { source } = &plan.sequences[0].tracks[0].clips[0].source
    else {
        unreachable!()
    };
    assert_eq!(source.angles.len(), 2);
    assert_eq!(source.switches, switches());
    assert_eq!(source.angles[1].source_offset, time(60));
    assert_eq!(source.sync.basis, MulticamSyncBasis::Manual);
    let hash = plan_hash(&plan).unwrap();
    let mut changed = project;
    let ClipSource::Multicam { switches, .. } =
        &mut changed.project.sequences[0].tracks[0].clips[0].source
    else {
        unreachable!()
    };
    switches[0].angle_id = MulticamAngleId::new("ang_wide").unwrap();
    assert_ne!(
        plan_hash(&resolve_one(&changed, &output_id).unwrap()).unwrap(),
        hash
    );
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
