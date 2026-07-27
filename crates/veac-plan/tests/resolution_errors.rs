mod support;

use support::*;
use veac_plan::{canonical::*, resolve, ResolutionErrorKind, ResolutionErrors};

fn assert_error(project: &ProjectEnvelope, code: &str, kind: ResolutionErrorKind) {
    let error = resolve(project, None).unwrap_err();
    let diagnostic = error
        .diagnostics()
        .iter()
        .find(|diagnostic| diagnostic.code == code)
        .unwrap_or_else(|| panic!("missing {code}; got {:?}", diagnostic_codes(&error)));
    assert_eq!(diagnostic.kind, kind);
}

#[test]
fn missing_identity_and_probe_are_typed() {
    let mut missing_identity = project();
    missing_identity.project.materials[0].identity = None;
    assert_error(
        &missing_identity,
        "MATERIAL_IDENTITY_MISSING",
        ResolutionErrorKind::MaterialIdentityMissing,
    );

    let mut missing_probe = project();
    missing_probe.project.materials[0].probe = None;
    assert_error(
        &missing_probe,
        "MATERIAL_PROBE_MISSING",
        ResolutionErrorKind::MaterialProbeMissing,
    );
}

#[test]
fn remote_and_family_font_require_external_resolution() {
    let mut remote = project();
    remote.project.materials[0] = remote_material("med_video");
    assert_error(
        &remote,
        "REMOTE_MATERIAL_UNRESOLVED",
        ResolutionErrorKind::RemoteMaterialUnresolved,
    );

    let mut family = project();
    let track = &mut family.project.sequences[0].tracks[0];
    track.kind = TrackKind::Visual;
    let clip = &mut track.clips[0];
    clip.source = ClipSource::Text {
        text: "title".to_owned(),
        style: text_style(FontRef::Family {
            family: "Inter".to_owned(),
        }),
    };
    clip.source_mapping = None;
    clip.visual = Some(visual_properties());
    let error = resolve(&family, None).unwrap_err();
    let diagnostic = &error.diagnostics()[0];
    assert_eq!(diagnostic.kind, ResolutionErrorKind::FontFamilyUnresolved);
    assert!(diagnostic.suggested_repair.is_some());
}

#[test]
fn required_audio_selection_is_not_inferred() {
    let mut project = project();
    project.project.render_configs[0]
        .video_deliverable_mut()
        .unwrap()
        .audio = Some(AudioOutput {
        codec: AudioCodec::Aac,
        sample_rate: 48_000,
        channels: 2,
    });
    project.project.sequences[0].tracks[0].clips[0].audio = Some(audio_properties());
    project.project.materials[0]
        .probe
        .as_mut()
        .unwrap()
        .selected_audio_stream = None;
    assert_error(
        &project,
        "REQUIRED_AUDIO_STREAM_MISSING",
        ResolutionErrorKind::RequiredStreamMissing,
    );
}

#[test]
fn source_duration_and_bounds_fail_before_codegen() {
    let mut missing_duration = project();
    let probe = missing_duration.project.materials[0]
        .probe
        .as_mut()
        .unwrap();
    probe.container_duration = None;
    probe.streams[0].duration = None;
    assert_error(
        &missing_duration,
        "SOURCE_DURATION_UNAVAILABLE",
        ResolutionErrorKind::SourceDurationUnavailable,
    );

    let mut outside = project();
    let probe = outside.project.materials[0].probe.as_mut().unwrap();
    probe.container_duration = Some(time(300));
    probe.streams[0].duration = Some(time(300));
    assert_error(
        &outside,
        "SOURCE_RANGE_OUT_OF_BOUNDS",
        ResolutionErrorKind::SourceRangeOutOfBounds,
    );
}

#[test]
fn explicit_selection_and_probe_identity_mismatch_keep_typed_origin() {
    let mut selection = project();
    selection.project.materials[0].stream_intent.video =
        StreamChoice::GlobalIndex { global_index: 9 };
    assert_error(
        &selection,
        "PROBE_INTENT",
        ResolutionErrorKind::StreamSelectionMismatch,
    );

    let mut identity_mismatch = project();
    identity_mismatch.project.materials[0].identity = Some(identity('c'));
    assert_error(
        &identity_mismatch,
        "PROBE_IDENTITY",
        ResolutionErrorKind::MaterialProbeIdentityMismatch,
    );
}

#[test]
fn source_time_overflow_and_unknown_output_are_structured() {
    let mut overflow = project();
    let clip = &mut overflow.project.sequences[0].tracks[0].clips[0];
    clip.record_range = range(0, 1200);
    let SourceTimeMap::Linear { rate, .. } = &mut clip.source_mapping.as_mut().unwrap().time_map
    else {
        panic!("linear mapping");
    };
    *rate = Rational::new(MAX_SAFE_INTEGER as i64, 1).unwrap();
    assert_error(
        &overflow,
        "SOURCE_TIME_ARITHMETIC",
        ResolutionErrorKind::TimeArithmetic,
    );

    let id = RenderConfigId::new("out_missing").unwrap();
    let error = veac_plan::resolve_one(&project(), &id).unwrap_err();
    assert_eq!(
        error.diagnostics()[0].kind,
        ResolutionErrorKind::RenderConfigNotFound
    );
    assert!(error.to_string().contains("RENDER_CONFIG_NOT_FOUND"));
    let owned = error.into_diagnostics();
    assert_eq!(owned.len(), 1);
}

#[test]
fn empty_error_collection_has_total_display() {
    let error = ResolutionErrors::new(Vec::new());
    assert_eq!(error.to_string(), "resolution failed without a diagnostic");
}
