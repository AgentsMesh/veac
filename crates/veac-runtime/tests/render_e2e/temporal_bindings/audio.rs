use std::process::Command;
use tempfile::tempdir;

use super::super::support::*;
use super::{install, scaled_progress};

#[test]
fn progress_binding_drives_real_audio_gain() {
    let temp = tempdir().unwrap();
    let source = tone_fixture(temp.path(), "temporal-tone", 440);
    let output = temp.path().join("temporal-audio.mp4");
    let mut project = project(true);
    project.project.materials.push(material(
        "med_temporal_tone",
        MaterialKind::Audio,
        StreamChoice::Disabled,
        StreamChoice::Auto,
    ));
    let binding = install(
        &mut project,
        "audio_gain",
        "itm_temporal_audio",
        TemporalType::Scalar,
        scaled_progress(1.0),
        2,
    );
    let mut tone = media_clip("itm_temporal_audio", "med_temporal_tone", 0, 1_000);
    let mut audio = audio_properties(1.0);
    audio.gain = Animatable::Binding {
        binding_id: binding,
    };
    tone.audio = Some(audio);
    project.project.sequences[0].tracks.extend([
        track(
            "trk_temporal_picture",
            TrackKind::Video,
            0,
            vec![solid_clip(
                "itm_temporal_picture",
                color(8, 12, 16),
                0,
                1_000,
            )],
        ),
        track("trk_temporal_audio", TrackKind::Audio, 1, vec![tone]),
    ]);
    render(
        project,
        &BTreeMap::from([("med_temporal_tone".to_owned(), source)]),
        &output,
    );

    let early = rms_db(&audio_samples(&output, 0.1, 0.2));
    let late = rms_db(&audio_samples(&output, 0.7, 0.2));
    assert!(late > early + 8.0, "gain ramp {early}..{late} dB");
}

#[test]
fn progress_binding_drives_real_stereo_pan() {
    let temp = tempdir().unwrap();
    let source = tone_fixture(temp.path(), "temporal-pan-tone", 440);
    let output = temp.path().join("temporal-pan.mp4");
    let mut project = project(true);
    let config = &mut project.project.render_configs[0];
    let deliverable = config.deliverables[0].id.clone();
    config
        .video_deliverable_mut(&deliverable)
        .unwrap()
        .audio
        .as_mut()
        .unwrap()
        .channels = 2;
    project.project.materials.push(material(
        "med_temporal_pan",
        MaterialKind::Audio,
        StreamChoice::Disabled,
        StreamChoice::Auto,
    ));
    let binding = install(
        &mut project,
        "audio_pan",
        "itm_temporal_pan",
        TemporalType::Scalar,
        scaled_progress(1.0),
        2,
    );
    let mut tone = media_clip("itm_temporal_pan", "med_temporal_pan", 0, 1_000);
    let mut audio = audio_properties(1.0);
    audio.pan = Animatable::Binding {
        binding_id: binding,
    };
    tone.audio = Some(audio);
    project.project.sequences[0].tracks.push(track(
        "trk_temporal_pan",
        TrackKind::Audio,
        0,
        vec![tone],
    ));
    render(
        project,
        &BTreeMap::from([("med_temporal_pan".to_owned(), source)]),
        &output,
    );

    let early_left = rms_db(&channel_samples(&output, 0.1, 0));
    let late_left = rms_db(&channel_samples(&output, 0.7, 0));
    let early_right = rms_db(&channel_samples(&output, 0.1, 1));
    let late_right = rms_db(&channel_samples(&output, 0.7, 1));
    assert!(
        early_left > late_left + 8.0,
        "left {early_left}..{late_left}"
    );
    assert!(
        (early_right - late_right).abs() < 2.0,
        "right {early_right}..{late_right}"
    );
}

fn channel_samples(media: &Path, start: f64, channel: usize) -> Vec<f64> {
    let filter = format!("pan=mono|c0=c{channel}");
    let output = Command::new("ffmpeg")
        .args(["-hide_banner", "-loglevel", "error", "-i"])
        .arg(media)
        .arg("-ss")
        .arg(start.to_string())
        .args(["-t", "0.2", "-map", "0:a:0", "-af", &filter])
        .args(["-ar", "8000", "-f", "s16le", "-"])
        .output()
        .expect("extract channel PCM");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
        .stdout
        .chunks_exact(2)
        .map(|bytes| f64::from(i16::from_le_bytes([bytes[0], bytes[1]])) / 32768.0)
        .collect()
}
