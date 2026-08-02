mod support;

use std::collections::BTreeSet;

use support::*;
use veac_plan::canonical::*;
use veac_plan::{required_material_ids_one, resolve_one};

#[test]
fn active_window_selects_exact_key_clips_without_projecting_a_second_hop() {
    let mut envelope = fixture(range(100, 100), range(100, 100));
    add_key_material(&mut envelope, "med_key_hit", true);
    add_key_material(&mut envelope, "med_key_miss", false);
    add_key_material(&mut envelope, "med_second_hop", false);
    let sequence = &mut envelope.project.sequences[0];
    sequence.tracks[1].clips.extend([
        audio_clip("itm_key_hit", "med_key_hit", 100),
        audio_clip("itm_key_miss", "med_key_miss", 400),
    ]);
    sequence.tracks[2]
        .clips
        .push(audio_clip("itm_second_hop", "med_second_hop", 100));
    add_sidechain(
        &mut envelope,
        "rel_second_hop",
        "seq_main",
        RelationEndpoint::track(TrackId::new("trk_second_hop").unwrap()),
        "itm_key_hit",
        parameters(range(0, 100)),
    );

    let config_id = envelope.project.render_configs[0].id.clone();
    let required = required_material_ids_one(&envelope, &config_id).unwrap();
    assert_eq!(required, ids(&["med_key_hit", "med_video"]));
    let plan = resolve_one(&envelope, &config_id).unwrap();
    assert_eq!(planned_ids(&plan), required);
    let key = clips(&plan)
        .find(|clip| clip.id.as_str() == "itm_key_hit")
        .unwrap();
    assert!(key.audio.as_ref().unwrap().sidechain.is_none());
    assert!(
        clips(&plan).all(|clip| { !matches!(clip.id.as_str(), "itm_key_miss" | "itm_second_hop") })
    );
}

#[test]
fn sidechain_with_no_overlapping_key_clip_is_a_plan_no_op() {
    let mut envelope = fixture(range(0, 100), range(300, 100));
    add_key_material(&mut envelope, "med_key_miss", false);
    envelope.project.sequences[0].tracks[1]
        .clips
        .push(audio_clip("itm_key_miss", "med_key_miss", 300));
    let config_id = envelope.project.render_configs[0].id.clone();
    assert_eq!(
        required_material_ids_one(&envelope, &config_id).unwrap(),
        ids(&["med_video"]),
    );
    let plan = resolve_one(&envelope, &config_id).unwrap();
    let target = clips(&plan)
        .find(|clip| clip.id.as_str() == "itm_video")
        .unwrap();
    assert!(target.audio.as_ref().unwrap().sidechain.is_none());
}

fn fixture(active: TimeRange, _key: TimeRange) -> ProjectEnvelope {
    let mut envelope = project();
    audio_stem(&mut envelope);
    envelope.project.sequences[0].tracks[0].clips[0].audio = Some(audio_properties());
    envelope.project.sequences[0].tracks.extend([
        track("trk_key", TrackKind::Audio, 1, Vec::new()),
        track("trk_second_hop", TrackKind::Audio, 2, Vec::new()),
    ]);
    add_sidechain(
        &mut envelope,
        "rel_target_key",
        "seq_main",
        RelationEndpoint::track(TrackId::new("trk_key").unwrap()),
        "itm_video",
        parameters(active),
    );
    envelope
}

fn audio_stem(envelope: &mut ProjectEnvelope) {
    let config = &mut envelope.project.render_configs[0];
    config.raster = None;
    config.deliverables = vec![Deliverable {
        id: DeliverableId::new("dlv_audio").unwrap(),
        target: DeliverableTarget::File {
            name: "audio.wav".to_owned(),
        },
        kind: DeliverableKind::AudioStem(AudioStemOutput {
            format: AudioStemFormat::Wav,
            audio: AudioOutput {
                codec: AudioCodec::PcmS16Le,
                sample_rate: 48_000,
                channels: 2,
            },
            source: AudioMixSource::Track {
                track_id: TrackId::new("trk_video").unwrap(),
            },
        }),
    }];
}

fn audio_clip(id: &str, material: &str, start: i64) -> Clip {
    let mut clip = media_clip(id, material, start);
    clip.record_range.duration = time(100);
    clip.audio = Some(audio_properties());
    clip
}

fn add_key_material(envelope: &mut ProjectEnvelope, id: &str, hydrated: bool) {
    let mut material = audio_material(id);
    if !hydrated {
        material.source = MaterialSource::File {
            uri: format!("media/{id}-missing.wav"),
        };
        material.identity = None;
        material.probe = None;
    }
    envelope.project.materials.push(material);
}

fn parameters(active_range: TimeRange) -> SidechainRelationParameters {
    SidechainRelationParameters {
        threshold_db: -18.0,
        ratio: 4.0,
        attack_ms: 10.0,
        release_ms: 200.0,
        active_range: Some(active_range),
    }
}

fn ids(values: &[&str]) -> BTreeSet<MaterialId> {
    values
        .iter()
        .map(|id| MaterialId::new(*id).unwrap())
        .collect()
}

fn planned_ids(plan: &veac_plan::ResolvedRenderPlan) -> BTreeSet<MaterialId> {
    plan.inputs
        .iter()
        .filter_map(|input| input.material_id.clone())
        .collect()
}

fn clips(plan: &veac_plan::ResolvedRenderPlan) -> impl Iterator<Item = &veac_plan::ResolvedClip> {
    plan.sequences
        .iter()
        .flat_map(|sequence| &sequence.tracks)
        .flat_map(|track| &track.clips)
}
