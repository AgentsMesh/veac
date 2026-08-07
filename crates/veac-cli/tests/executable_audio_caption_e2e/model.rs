use veac_ir::{ClipSource, Material, MaterialKind, ProjectEnvelope, TrackKind};
use veac_plan::{ResolvedClipSource, ResolvedInput, ResolvedInputKind, ResolvedRenderPlan};

use super::fixture::{FONT_SHA, TONE_SHA};

pub fn assert_canonical(envelope: &ProjectEnvelope) {
    let audio = material(envelope, MaterialKind::Audio);
    let font = material(envelope, MaterialKind::Font);
    assert_identity(audio, TONE_SHA);
    assert_identity(font, FONT_SHA);
    assert!(audio.probe.is_none() && font.probe.is_none());

    let captions = envelope.project.sequences[0]
        .tracks
        .iter()
        .find(|track| track.kind == TrackKind::Caption)
        .unwrap();
    let ClipSource::Caption { speaker, .. } = &captions.clips[0].source else {
        panic!("expected canonical caption")
    };
    assert_eq!(speaker.as_deref(), Some("小石"));
}

pub fn assert_plan(plan: &ResolvedRenderPlan) {
    let tone = input(plan, "assets/tone.wav");
    let font = input(plan, "assets/veac-example-zh.ttf");
    assert_eq!(tone.observed_identity.digest, TONE_SHA);
    assert_eq!(font.observed_identity.digest, FONT_SHA);
    assert!(tone.probe.is_some() && tone.audio.is_some());
    assert!(font.probe.is_none());
    assert!(matches!(font.kind, ResolvedInputKind::Font { .. }));

    let captions = plan.sequences[0]
        .tracks
        .iter()
        .find(|track| track.kind == TrackKind::Caption)
        .unwrap();
    let ResolvedClipSource::Caption { speaker, .. } = &captions.clips[0].source else {
        panic!("expected resolved caption")
    };
    assert_eq!(speaker.as_deref(), Some("小石"));
}

fn material(envelope: &ProjectEnvelope, kind: MaterialKind) -> &Material {
    envelope
        .project
        .materials
        .iter()
        .find(|material| material.kind == kind)
        .unwrap()
}

fn assert_identity(material: &Material, digest: &str) {
    assert_eq!(material.identity.as_ref().unwrap().digest, digest);
}

fn input<'a>(plan: &'a ResolvedRenderPlan, uri: &str) -> &'a ResolvedInput {
    plan.inputs
        .iter()
        .find(|input| input.canonical_uri == uri)
        .unwrap()
}
