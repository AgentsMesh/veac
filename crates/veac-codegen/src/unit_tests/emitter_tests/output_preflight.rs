use veac_plan::canonical::*;
use veac_plan::{PlanInputId, ResolvedClipSource};

use super::support::{bindings, emit_video_command, fixture, resolved, time};

#[test]
fn emits_every_compatible_container_video_and_audio_codec() {
    let cases = [
        (
            OutputFormat::Mp4,
            VideoCodec::H264,
            AudioCodec::Aac,
            "mp4",
            "libx264",
            "aac",
        ),
        (
            OutputFormat::Mov,
            VideoCodec::H265,
            AudioCodec::PcmS16Le,
            "mov",
            "libx265",
            "pcm_s16le",
        ),
        (
            OutputFormat::Mkv,
            VideoCodec::Vp9,
            AudioCodec::Flac,
            "matroska",
            "libvpx-vp9",
            "flac",
        ),
        (
            OutputFormat::Webm,
            VideoCodec::Av1,
            AudioCodec::Opus,
            "webm",
            "libaom-av1",
            "libopus",
        ),
    ];
    for (format, video, audio, container, video_encoder, audio_encoder) in cases {
        let mut plan = resolved(&fixture());
        plan.output.deliverables[0].file_name = format!("output.{}", extension(format));
        let delivery = delivery(&mut plan);
        delivery.container = format;
        delivery.video.codec = video;
        delivery.audio = Some(AudioOutput {
            codec: audio,
            sample_rate: 48_000,
            channels: 6,
        });
        let command = emit_video_command(&plan, &bindings(&plan)).unwrap();
        let args = command.output_args;
        assert!(pair(&args, "-f", container));
        assert!(pair(&args, "-c:v", video_encoder));
        assert!(pair(&args, "-c:a", audio_encoder));
        assert!(pair(&args, "-ac", "6"));
        assert!(command.filter_graph.unwrap().contains("cl=6c"));
    }

    let plan = resolved(&fixture());
    let command = emit_video_command(&plan, &bindings(&plan)).unwrap();
    assert!(command.output_args.contains(&"-an".to_owned()));
}

#[test]
fn rejects_every_container_codec_incompatibility() {
    let video_cases = [
        (OutputFormat::Mp4, VideoCodec::Vp9),
        (OutputFormat::Mov, VideoCodec::Av1),
        (OutputFormat::Webm, VideoCodec::H264),
    ];
    for (format, codec) in video_cases {
        let mut plan = resolved(&fixture());
        let delivery = delivery(&mut plan);
        delivery.container = format;
        delivery.video.codec = codec;
        assert!(codes(&plan).contains(&"PLAN_VIDEO_CODEC_INCOMPATIBLE"));
    }
    let audio_cases = [
        (OutputFormat::Mp4, VideoCodec::H264, AudioCodec::Opus),
        (OutputFormat::Mov, VideoCodec::H264, AudioCodec::Flac),
        (OutputFormat::Webm, VideoCodec::Vp9, AudioCodec::Aac),
    ];
    for (format, video, audio) in audio_cases {
        let mut plan = resolved(&fixture());
        let delivery = delivery(&mut plan);
        delivery.container = format;
        delivery.video.codec = video;
        delivery.audio = Some(AudioOutput {
            codec: audio,
            sample_rate: 48_000,
            channels: 2,
        });
        assert!(codes(&plan).contains(&"PLAN_AUDIO_CODEC_INCOMPATIBLE"));
    }
}

#[test]
fn preflight_reports_entry_sequence_and_freeze_reference_failures() {
    let mut plan = resolved(&fixture());
    plan.entry_sequence_id = SequenceId::new("seq_absent").unwrap();
    let found = codes(&plan);
    assert!(found.contains(&"PLAN_ENTRY_MISSING"));
    assert!(found.contains(&"PLAN_ENTRY_MISMATCH"));

    let mut plan = resolved(&fixture());
    plan.sequences[0].tracks[0].clips[0].source = ResolvedClipSource::Sequence {
        sequence_id: SequenceId::new("seq_absent").unwrap(),
    };
    assert!(codes(&plan).contains(&"PLAN_SEQUENCE_MISSING"));

    let mut plan = resolved(&fixture());
    plan.sequences[0].tracks[0].clips[0].source = ResolvedClipSource::FreezeFrame {
        input_id: PlanInputId::new("pin_absent").unwrap(),
        video_stream: StreamSelection {
            global_index: 2,
            type_index: 0,
        },
        source_time: time(0),
    };
    assert!(codes(&plan).contains(&"PLAN_INPUT_MISSING"));
}

#[test]
fn preflight_rejects_audio_channel_overflow_and_output_mismatch() {
    let mut plan = resolved(&fixture());
    delivery(&mut plan).audio = Some(AudioOutput {
        codec: AudioCodec::Aac,
        sample_rate: 48_000,
        channels: 33,
    });
    assert!(codes(&plan).contains(&"PLAN_AUDIO_OUTPUT_INVALID"));
}

fn delivery(plan: &mut veac_plan::ResolvedRenderPlan) -> &mut VideoDeliverable {
    plan.output.video_deliverable_mut().unwrap()
}

fn codes(plan: &veac_plan::ResolvedRenderPlan) -> Vec<&'static str> {
    emit_video_command(plan, &bindings(plan))
        .unwrap_err()
        .diagnostics()
        .iter()
        .map(|diagnostic| diagnostic.code)
        .collect()
}

fn pair(args: &[String], name: &str, value: &str) -> bool {
    args.windows(2).any(|pair| pair == [name, value])
}

fn extension(format: OutputFormat) -> &'static str {
    match format {
        OutputFormat::Mp4 => "mp4",
        OutputFormat::Mov => "mov",
        OutputFormat::Mkv => "mkv",
        OutputFormat::Webm => "webm",
        OutputFormat::Mxf => "mxf",
    }
}
