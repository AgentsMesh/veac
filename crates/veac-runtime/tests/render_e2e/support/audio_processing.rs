use std::process::Command;

use super::*;

pub(crate) fn tone_wav(
    directory: &Path,
    name: &str,
    frequency: u32,
    duration: f64,
    volume: f64,
) -> PathBuf {
    let output = directory.join(format!("{name}.m4a"));
    let source =
        format!("sine=frequency={frequency}:sample_rate=48000:duration={duration},volume={volume}");
    run(Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
            &source,
        ])
        .args(["-c:a", "aac", "-b:a", "192k", "-ac", "1"])
        .arg(&output));
    output
}

pub(crate) fn dual_tone_wav(directory: &Path, name: &str, first: u32, second: u32) -> PathBuf {
    let output = directory.join(format!("{name}.m4a"));
    let first = format!("sine=frequency={first}:sample_rate=48000:duration=1");
    let second = format!("sine=frequency={second}:sample_rate=48000:duration=1");
    run(Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
            &first,
        ])
        .args(["-f", "lavfi", "-i", &second])
        .args([
            "-filter_complex",
            "[0:a][1:a]amix=inputs=2:normalize=0",
            "-c:a",
            "aac",
            "-b:a",
            "192k",
            "-ac",
            "1",
        ])
        .arg(&output));
    output
}

pub(crate) fn render_audio_chain(
    source: &Path,
    duration_ms: i64,
    processors: Vec<AudioProcessor>,
    crossfade: Option<AudioCrossfade>,
    output: &Path,
) -> Rendered {
    let mut canonical = project(true);
    canonical.project.materials.push(material(
        "med_audio_process",
        MaterialKind::Audio,
        StreamChoice::Disabled,
        StreamChoice::Auto,
    ));
    let mut clip = media_clip("itm_audio_process", "med_audio_process", 0, duration_ms);
    let mut audio = audio_properties(1.0);
    audio.processors = processors;
    audio.crossfade = crossfade;
    clip.audio = Some(audio);
    canonical.project.sequences[0].tracks.push(track(
        "trk_audio_process",
        TrackKind::Audio,
        0,
        vec![clip],
    ));
    render(
        canonical,
        &BTreeMap::from([("med_audio_process".to_owned(), source.to_path_buf())]),
        output,
    )
}

pub(crate) fn identified_processor(id: &str, kind: AudioProcessorKind) -> AudioProcessor {
    AudioProcessor {
        id: AudioProcessorId::new(format!("aud_{id}")).unwrap(),
        kind,
    }
}

pub(crate) fn peak(samples: &[f64]) -> f64 {
    samples.iter().copied().map(f64::abs).fold(0.0, f64::max)
}

pub(crate) fn run(command: &mut Command) {
    let output = command.output().expect("start FFmpeg fixture command");
    assert!(
        output.status.success(),
        "FFmpeg fixture failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
