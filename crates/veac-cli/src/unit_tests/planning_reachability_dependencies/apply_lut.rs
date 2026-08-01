use tempfile::tempdir;
use veac_ir::{
    Apply, ApplyId, ApplyMix, ApplyOperation, ApplyStage, ApplyStageId, ApplyTarget, ColorMatrix,
    ColorPipeline, ColorPrimaries, ColorRange, ColorSpace, ColorStage, ColorTransfer, ItemId,
    LutApplication, LutInterpolation, Material, MaterialId, MaterialKind, MaterialSource,
    PlacementMode, RationalTime, StreamChoice, StreamIntent, TimeRange, TrackId,
};

use super::super::support::{
    assert_project_inputs, canonical_project, write_project, FakeEnvironment, MEDIA_SOURCE,
};

#[test]
fn apply_lut_hydration_tracks_the_target_activity_window() {
    let temp = tempdir().unwrap();
    std::fs::write(temp.path().join("clip.mp4"), b"fixture").unwrap();
    std::fs::write(temp.path().join("active.cube"), b"lut").unwrap();
    let project = canonical_project(&temp, MEDIA_SOURCE);
    let mut envelope = crate::canonical::load(&project).unwrap();
    let base = envelope.project.materials[0].clone();
    envelope.project.materials.extend([
        lut(&base, "med_lut_active", "active.cube"),
        lut(
            &base,
            "med_lut_inactive",
            "definitely-missing-inactive.cube",
        ),
    ]);
    let sequence = &mut envelope.project.sequences[0];
    let target_track_id = sequence.tracks[0].id.clone();
    sequence.tracks[0].clips[0].record_range.duration = time(300);
    let mut extension = sequence.tracks[0].clips[0].clone();
    extension.id = ItemId::new("itm_extension").unwrap();
    extension.record_range = range(500, 100);
    extension.audio = None;
    let mut extension_track = sequence.tracks[0].clone();
    extension_track.id = TrackId::new("trk_extension").unwrap();
    extension_track.order = 1;
    extension_track.placement_mode = PlacementMode::Free;
    extension_track.clips = vec![extension];
    sequence.tracks.push(extension_track);
    sequence.applies.push(Apply {
        id: ApplyId::new("apl_grade").unwrap(),
        enabled: true,
        record_range: range(0, 600),
        target: ApplyTarget::Layer {
            track_id: target_track_id,
        },
        stages: vec![
            stage("aps_active", range(0, 100), "med_lut_active"),
            stage("aps_inactive", range(400, 100), "med_lut_inactive"),
        ],
        mix: ApplyMix::default(),
    });

    assert_project_inputs(
        &project,
        &envelope,
        &["med_footage", "med_lut_active"],
        &["active.cube"],
        &["clip.mp4"],
    );
    envelope.project.sequences[0].applies[0].stages[1].active_range = Some(range(200, 100));
    write_project(&project, &envelope);
    let error = crate::planning::prepare(&project, None, &FakeEnvironment::success()).unwrap_err();
    assert!(error.to_string().contains("PATH_UNAVAILABLE"));
}

fn lut(base: &Material, id: &str, uri: &str) -> Material {
    let mut material = base.clone();
    material.id = MaterialId::new(id).unwrap();
    material.kind = MaterialKind::Lut3d;
    material.source = MaterialSource::File { uri: uri.into() };
    material.identity = None;
    material.probe = None;
    material.stream_intent = StreamIntent {
        video: StreamChoice::Disabled,
        audio: StreamChoice::Disabled,
    };
    material
}

fn stage(id: &str, active_range: TimeRange, material: &str) -> ApplyStage {
    ApplyStage {
        id: ApplyStageId::new(id).unwrap(),
        enabled: true,
        active_range: Some(active_range),
        operation: ApplyOperation::Color {
            pipeline: ColorPipeline {
                input: rec709(),
                working: rec709(),
                output: rec709(),
                stages: vec![ColorStage::Lut {
                    application: LutApplication {
                        material_id: MaterialId::new(material).unwrap(),
                        interpolation: LutInterpolation::Tetrahedral,
                    },
                }],
            },
        },
    }
}

fn rec709() -> ColorSpace {
    ColorSpace {
        primaries: ColorPrimaries::Bt709,
        transfer: ColorTransfer::Bt709,
        matrix: ColorMatrix::Bt709,
        range: ColorRange::Limited,
    }
}

fn time(value: i64) -> RationalTime {
    RationalTime::new(value, 1000).unwrap()
}

fn range(start: i64, duration: i64) -> TimeRange {
    TimeRange::new(time(start), time(duration)).unwrap()
}
