use crate::test_support::{multicam_project, sample_project};

use super::*;

#[test]
fn material_queries_cover_media_font_freeze_and_multicam_references() {
    let project = sample_project();
    let video = MaterialId::new("med_video").unwrap();
    let font = MaterialId::new("med_font").unwrap();
    let missing = MaterialId::new("med_missing").unwrap();
    let clips: Vec<_> = project
        .project
        .sequences
        .iter()
        .flat_map(|sequence| &sequence.tracks)
        .flat_map(|track| &track.clips)
        .collect();
    assert!(clips.iter().any(|clip| clip_uses_material(clip, &video)));
    assert!(clips.iter().any(|clip| clip_uses_material(clip, &font)));
    assert!(material_is_referenced(&project.project, &video));
    assert!(material_is_referenced(&project.project, &font));
    assert!(!material_is_referenced(&project.project, &missing));

    let mut freeze = clips[0].clone();
    freeze.source = ClipSource::FreezeFrame {
        material_id: video.clone(),
        source_time: RationalTime::zero(project.project.timebase).unwrap(),
    };
    assert!(clip_uses_material(&freeze, &video));

    let mut multicam = multicam_project();
    assert!(material_is_referenced(&multicam.project, &video));
    assert!(!material_is_used_on_locked_track(&multicam.project, &video));
    multicam.project.sequences[0].tracks[0].state.locked = true;
    assert!(material_is_used_on_locked_track(&multicam.project, &video));
    assert!(!material_is_used_on_locked_track(
        &multicam.project,
        &missing
    ));
}

#[test]
fn sequence_and_multicam_queries_cover_every_reference_owner() {
    let mut project = sample_project().project;
    let main = project.entry_sequence_id.clone();
    let missing = SequenceId::new("seq_missing").unwrap();
    assert!(sequence_is_referenced(&project, &main));
    assert!(!sequence_is_referenced(&project, &missing));

    let auxiliary = SequenceId::new("seq_auxiliary").unwrap();
    project.entry_sequence_id = auxiliary.clone();
    project.render_configs[0].sequence_id = auxiliary;
    project.sequences[0].tracks[1].clips[0].source = ClipSource::Sequence {
        sequence_id: main.clone(),
    };
    assert!(sequence_is_referenced(&project, &main));

    let mut multicam = multicam_project().project;
    let group = multicam.multicam_groups[0].id.clone();
    let absent = MulticamGroupId::new("mcg_absent").unwrap();
    assert!(multicam_group_is_referenced(&multicam, &group));
    assert!(!multicam_group_is_referenced(&multicam, &absent));
    assert!(!multicam_group_is_used_on_locked_track(&multicam, &group));
    multicam.sequences[0].tracks[0].state.locked = true;
    assert!(multicam_group_is_used_on_locked_track(&multicam, &group));
    assert!(!multicam_group_is_used_on_locked_track(&multicam, &absent));
}
