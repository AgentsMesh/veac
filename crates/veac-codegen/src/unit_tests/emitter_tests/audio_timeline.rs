use veac_plan::canonical::{AudioProcessor, AudioProcessorId, AudioProcessorKind, LoudnessTarget};

use super::audio::{audio_plan, clip};
use super::support::{bindings, emit_video_command, time};

#[test]
fn processed_audio_returns_to_output_rate_before_sample_delay() {
    let mut plan = audio_plan(2);
    let clip = clip(&mut plan);
    clip.record_range.start = time(300);
    clip.audio.as_mut().unwrap().processors = vec![AudioProcessor {
        id: AudioProcessorId::new("aud_target-loudness").unwrap(),
        kind: AudioProcessorKind::Loudness(LoudnessTarget {
            integrated_lufs: -16.0,
            true_peak_dbtp: -1.0,
            loudness_range_lu: 7.0,
        }),
    }];
    plan.sequences[0].duration = time(900);

    let graph = emit_video_command(&plan, &bindings(&plan))
        .unwrap()
        .filter_graph
        .unwrap();
    assert!(graph.contains("loudnorm=I=-16:TP=-1:LRA=7"));
    assert!(graph.contains("aresample=48000,adelay=delays=24000S:all=1"));
    assert!(graph.contains("aresample=48000,asetnsamples=n=1024:p=1,asetpts=N/SR/TB"));
}
