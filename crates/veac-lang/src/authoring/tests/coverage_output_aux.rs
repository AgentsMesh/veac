use super::coverage_output_support::output;

#[test]
fn every_image_sequence_format_preserves_format_and_start_number() {
    use veac_ir::ImageFormat as F;
    for (token, expected) in [
        ("png", F::Png),
        ("jpeg", F::Jpeg),
        ("tiff", F::Tiff),
        ("exr", F::Exr),
    ] {
        let veac_ir::DeliverableKind::ImageSequence(settings) = output(
            "image-sequence",
            &format!("numbering from 1001; encode {token};"),
        ) else {
            panic!("image sequence expected")
        };
        assert_eq!((settings.format, settings.start_number), (expected, 1001));
    }
}

#[test]
fn every_caption_sidecar_format_preserves_the_selected_track() {
    use veac_ir::CaptionSidecarFormat as F;
    for (token, expected) in [("srt", F::Srt), ("web-vtt", F::WebVtt), ("ass", F::Ass)] {
        let veac_ir::DeliverableKind::CaptionSidecar(settings) = output(
            "caption-sidecar",
            &format!("source caption-tracks {{ track captions; }} encode {token};"),
        ) else {
            panic!("caption sidecar expected")
        };
        assert_eq!(settings.format, expected);
        assert_eq!(settings.track_ids[0].as_str(), "trk_captions");
    }
}

#[test]
fn audio_stem_formats_and_all_source_kinds_lower_exactly() {
    use veac_ir::{AudioCodec, AudioMixSource, AudioStemFormat, DeliverableKind};
    let cases = [
        (
            "source master; encode wav { sample-format pcm-s16le; sample-rate 48khz; channel-layout mono; }",
            AudioStemFormat::Wav,
            "master",
        ),
        (
            "source track dialogue; encode wav { sample-format pcm-s24le; sample-rate 48khz; channel-layout stereo; }",
            AudioStemFormat::Wav,
            "track",
        ),
        (
            "source bus dialogue; encode flac { sample-rate 96khz; channel-layout surround-5-1; }",
            AudioStemFormat::Flac,
            "bus",
        ),
    ];
    for (body, format, source) in cases {
        let DeliverableKind::AudioStem(settings) = output("audio-stem", body) else {
            panic!("audio stem expected")
        };
        assert_eq!(settings.format, format);
        match (source, settings.source) {
            ("master", AudioMixSource::Master) => {}
            ("track", AudioMixSource::Track { track_id }) => {
                assert_eq!(track_id.as_str(), "trk_dialogue");
            }
            ("bus", AudioMixSource::Bus { bus_id }) => {
                assert_eq!(bus_id.as_str(), "bus_dialogue");
            }
            value => panic!("unexpected stem source: {value:?}"),
        }
        assert!(matches!(
            settings.audio.codec,
            AudioCodec::PcmS16Le | AudioCodec::PcmS24Le | AudioCodec::Flac
        ));
    }
}

#[test]
fn scope_types_and_image_formats_lower_with_exact_geometry_and_time() {
    use veac_ir::{ImageFormat as F, VideoScope as S};
    for (scope, image, expected_scope, expected_image) in [
        ("waveform", "png", S::Waveform, F::Png),
        ("vectorscope", "jpeg", S::Vectorscope, F::Jpeg),
        ("histogram", "tiff", S::Histogram, F::Tiff),
    ] {
        let veac_ir::DeliverableKind::Scope(settings) = output(
            "scope",
            &format!(
                "analyze {scope}; frame containing 500ms; canvas 640px by 360px; encode {image};"
            ),
        ) else {
            panic!("scope expected")
        };
        assert_eq!(
            (settings.scope, settings.format),
            (expected_scope, expected_image)
        );
        assert_eq!((settings.width, settings.height), (640, 360));
        assert_eq!((settings.at.value, settings.at.timescale), (500, 1000));
    }
}
