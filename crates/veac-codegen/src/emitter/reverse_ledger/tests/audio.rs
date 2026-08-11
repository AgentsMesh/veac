use veac_plan::canonical::*;
use veac_plan::{ResolvedClipSource, ResolvedSidechain};

use super::support::*;

#[test]
fn audio_uses_output_rate_and_effective_source_channels() {
    let mut plan = audio_plan(384_000, 2);
    plan.inputs[0].audio.as_mut().unwrap().info.channels = 64;
    set_reverse_span(&mut plan, 3_600);
    assert_eq!(codes(&plan), ["PLAN_BUDGET_REVERSE_BYTES"]);

    let proxy = proxy_audio_bindings(&plan, 48_000, 2);
    assert!(codes_with(&plan, &proxy).is_empty());
}

#[test]
fn video_and_audio_reverse_bytes_share_one_delivery_ledger() {
    let mut project = fixture();
    project.project.sequences[0].tracks[0].clips[0].audio = Some(authored_audio_properties());
    let DeliverableKind::Video(video) = &mut project.project.render_configs[0].deliverables[0].kind
    else {
        unreachable!()
    };
    video.audio = Some(AudioOutput {
        codec: AudioCodec::Aac,
        sample_rate: 48_000,
        channels: 2,
    });
    let mut plan = resolved(&project);
    reverse(&mut plan);
    plan.inputs[0].audio.as_mut().unwrap().info.channels = 64;

    assert_eq!(codes(&plan), ["PLAN_BUDGET_REVERSE_BYTES"]);
    let DeliverableKind::Video(video) = &mut plan.output.deliverables[0].kind else {
        unreachable!()
    };
    video.audio = None;
    assert!(codes(&plan).is_empty());
}

#[test]
fn main_mix_and_each_sidechain_target_rebuild_the_reverse_source() {
    let mut plan = audio_plan(384_000, 2);
    plan.inputs[0].audio.as_mut().unwrap().info.channels = 64;
    reverse(&mut plan);
    assert!(codes(&plan).is_empty());

    let entry = plan.sequences.last_mut().unwrap();
    let source_track = entry.tracks[0].id.clone();
    for (index, suffix) in ["a", "b"].into_iter().enumerate() {
        let mut target = entry.tracks[0].clone();
        target.id = TrackId::new(format!("trk_reverse_target_{suffix}")).unwrap();
        target.order = index as i32 + 1;
        target.source_order = index as u32 + 1;
        target.kind = TrackKind::Audio;
        target.state.visual_enabled = false;
        target.routing.visual = None;
        target.clips[0].id = ItemId::new(format!("itm_reverse_target_{suffix}")).unwrap();
        target.clips[0].source = ResolvedClipSource::Generated {
            generator: Generator::Silence,
        };
        target.clips[0].source_mapping = None;
        target.clips[0].visual = None;
        target.clips[0].audio.as_mut().unwrap().sidechain = Some(ResolvedSidechain {
            relation_id: RelationId::new(format!("rel_reverse_target_{suffix}")).unwrap(),
            source: SidechainSource::Track {
                track_id: source_track.clone(),
            },
            threshold_db: -24.0,
            ratio: 4.0,
            attack_ms: 10.0,
            release_ms: 250.0,
            active_range: None,
        });
        entry.tracks.push(target);
    }

    assert_eq!(codes(&plan), ["PLAN_BUDGET_REVERSE_BYTES"]);
}

#[test]
fn missing_audio_facts_fail_at_the_emission_wrapper() {
    let plan = audio_plan(48_000, 2);
    let bindings = crate::unit_tests::emitter_tests::support::bindings(&plan);
    let deliverable = &plan.output.deliverables[0];
    let mut context =
        crate::emitter::EmitContext::new_audio(&plan, &bindings, deliverable).unwrap();
    let error = context
        .reverse_audio(
            "raw",
            "reversea",
            &plan.sequences[0].tracks[0].clips[0],
            time(1),
            48_000,
            None,
        )
        .unwrap_err();
    assert_eq!(
        error.diagnostics()[0].code,
        "PLAN_REVERSE_SOURCE_FACTS_MISSING"
    );
}

fn authored_audio_properties() -> AudioProperties {
    AudioProperties {
        gain: Animatable::constant(1.0),
        pan: Animatable::constant(0.0),
        muted: false,
        normalize: false,
        pitch_policy: PitchPolicy::Preserve,
        processors: vec![],
        crossfade: None,
    }
}
