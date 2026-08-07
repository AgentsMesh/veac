use veac_ir::{AudioCodec, CaptionOutput};

use super::super::support;
use super::SOURCE;

#[test]
fn remaining_audio_codecs_channels_and_caption_mode_lower() {
    let cases = [
        ("audio_pcm_s16le()", AudioCodec::PcmS16Le),
        ("audio_pcm_s24le()", AudioCodec::PcmS24Le),
        ("audio_pcm_s32le()", AudioCodec::PcmS32Le),
    ];
    for (codec, expected) in cases {
        let source = SOURCE.replace("audio_pcm_s24le()", codec);
        let envelope = support::envelope(&source);
        let audio = envelope.project.render_configs[0]
            .deliverables
            .iter()
            .find_map(|value| match &value.kind {
                veac_ir::DeliverableKind::AudioStem(stem) => Some(&stem.audio),
                _ => None,
            })
            .unwrap();
        assert_eq!(audio.codec, expected);
    }
    let source = SOURCE
        .replace("master.wav", "master.flac")
        .replace("stem_wav()", "stem_flac()")
        .replace("audio_pcm_s24le()", "audio_flac()");
    let envelope = support::envelope(&source);
    assert!(envelope.project.render_configs[0]
        .deliverables
        .iter()
        .any(|value| matches!(
            &value.kind,
            veac_ir::DeliverableKind::AudioStem(stem) if stem.audio.codec == AudioCodec::Flac
        )));
    let source = SOURCE.replace("caption_discard()", "caption_burn_in()");
    let envelope = support::envelope(&source);
    assert_eq!(
        envelope.project.render_configs[0]
            .raster
            .as_ref()
            .unwrap()
            .captions,
        CaptionOutput::BurnIn
    );
    let source = SOURCE.replace("channel_stereo()", "channel_mono()");
    assert!(support::envelope(&source).project.render_configs[0]
        .deliverables
        .iter()
        .any(|value| matches!(
            &value.kind,
            veac_ir::DeliverableKind::AudioFile(mp3)
                if matches!(
                    mp3.encoding,
                    veac_ir::AudioFileEncoding::Mp3(ref settings)
                        if settings.channel_layout == veac_ir::AudioChannelLayout::Mono
                )
        )));
}

#[test]
fn track_and_bus_mix_sources_lower_to_stable_references() {
    let source = SOURCE
        .replace(
            "let visual = visual_layer(",
            "let bus = audio_bus(identifier(\"stem\"));\n  let visual = visual_layer(",
        )
        .replace(
            "let captions = caption_layer(",
            "let sound = audio_layer(identifier(\"sound\"), 2, placement_free(), state, \
             track_routing_bus(bus));\n  let captions = caption_layer(",
        )
        .replace(
            ").with_layer(visual).with_layer(captions);",
            ").with_layer(visual).with_layer(captions).with_layer(sound);",
        )
        .replacen("mix_master()", "mix_layer(sound)", 1)
        .replacen("mix_master()", "mix_bus(bus)", 1);
    let envelope = support::envelope(&source);
    let deliverables = &envelope.project.render_configs[0].deliverables;
    assert!(deliverables.iter().any(|value| matches!(
        &value.kind,
        veac_ir::DeliverableKind::AudioStem(stem)
            if matches!(stem.source, veac_ir::AudioMixSource::Track { .. })
    )));
    assert!(deliverables.iter().any(|value| matches!(
        &value.kind,
        veac_ir::DeliverableKind::AudioFile(mp3)
            if matches!(mp3.source, veac_ir::AudioMixSource::Bus { .. })
    )));
}

#[test]
fn delivery_integer_and_dimension_boundaries_fail_during_lowering() {
    let cases = [
        ("canvas(640px, 360px)", "canvas(640.5px, 360px)"),
        ("frame_rate(30, 1)", "frame_rate(0, 1)"),
        ("frame_rate(30, 1)", "frame_rate(30, 0)"),
        (
            "audio_output(audio_aac(), 48000, 2)",
            "audio_output(audio_aac(), 0, 2)",
        ),
        (
            "audio_output(audio_aac(), 48000, 2)",
            "audio_output(audio_aac(), 48000, 0)",
        ),
        (
            "audio_output(audio_aac(), 48000, 2)",
            "audio_output(audio_aac(), 48000, 256)",
        ),
        ("mix_master(), 192000", "mix_master(), 0"),
        ("image_png(), 1", "image_png(), 4294967296"),
        ("canvas(320px, 180px)", "canvas(4294967296px, 180px)"),
        ("800000, 1000000, 2000000", "-1, 1000000, 2000000"),
        ("b_frames_count(2)", "b_frames_count(256)"),
        ("gif_forever()", "gif_times(70000)"),
    ];
    for (from, to) in cases {
        let error = support::error(&SOURCE.replacen(from, to, 1));
        assert_eq!(error.code, "PROGRAM_EXECUTABLE_LOWER", "{to}");
    }
}
