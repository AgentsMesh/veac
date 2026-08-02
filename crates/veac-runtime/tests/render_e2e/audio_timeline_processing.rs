use tempfile::tempdir;
use veac_ir::StreamChoice;

use super::support::*;

#[test]
fn delayed_source_ranges_survive_loudness_processing() {
    let temp = tempdir().unwrap();
    let source = tone_wav(temp.path(), "timeline-loudness", 440, 8.0, 0.2);
    let mut canonical = project(true);
    canonical.project.materials.push(material(
        "med_timeline_loudness",
        MaterialKind::Audio,
        StreamChoice::Disabled,
        StreamChoice::Auto,
    ));
    canonical.project.sequences[0].tracks.push(track(
        "trk_picture",
        TrackKind::Video,
        0,
        vec![solid_clip("itm_picture", color(20, 40, 80), 0, 8_000)],
    ));

    let mut loudness = ranged_clip("itm_loudness", 4_000);
    loudness.audio.as_mut().unwrap().processors = vec![identified_processor(
        "timeline-loudness",
        AudioProcessorKind::Loudness(LoudnessTarget {
            integrated_lufs: -16.0,
            true_peak_dbtp: -1.0,
            loudness_range_lu: 7.0,
        }),
    )];
    let mut normalize = ranged_clip("itm_normalize", 6_000);
    normalize.effects.push(EffectInstance {
        id: EffectId::new("fx_normalize").unwrap(),
        effect_type: "audio.normalize".to_owned(),
        enabled: true,
        enable_range: None,
        parameters: BTreeMap::from([(
            "target_lufs".to_owned(),
            ParameterValue::Number { value: -18.0 },
        )]),
    });
    canonical.project.sequences[0].tracks.push(track(
        "trk_processed",
        TrackKind::Audio,
        1,
        vec![loudness, normalize],
    ));

    let output = temp.path().join("timeline-loudness.mp4");
    let rendered = render(
        canonical,
        &BTreeMap::from([("med_timeline_loudness".to_owned(), source)]),
        &output,
    );
    for second in [4.5, 6.5] {
        let level = rms_db(&audio_samples(&output, second, 0.5));
        assert!(
            level > -35.0,
            "audio at {second}s is silent ({level} dB); graph={}",
            rendered.command.filter_graph.as_deref().unwrap_or("<none>")
        );
    }
}

fn ranged_clip(id: &str, start_ms: i64) -> Clip {
    let mut clip = media_clip(id, "med_timeline_loudness", start_ms, 2_000);
    clip.source_mapping = Some(SourceMapping::linear(time(start_ms), ratio(1, 1)));
    clip.audio = Some(audio_properties(1.0));
    clip
}
