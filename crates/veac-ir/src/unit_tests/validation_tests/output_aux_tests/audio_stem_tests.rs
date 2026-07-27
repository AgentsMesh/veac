use super::*;

#[test]
fn master_track_and_bus_stems_accept_typed_format_codec_pairs() {
    assert_valid(stem(
        "master.WAV",
        AudioStemFormat::Wav,
        AudioCodec::PcmS16Le,
        AudioStemSource::Master,
    ));
    assert_valid(stem(
        "master.FLAC",
        AudioStemFormat::Flac,
        AudioCodec::Flac,
        AudioStemSource::Master,
    ));
    assert_valid(stem(
        "video.wav",
        AudioStemFormat::Wav,
        AudioCodec::PcmS16Le,
        AudioStemSource::Track {
            track_id: TrackId::new("trk_video").unwrap(),
        },
    ));

    let value = stem(
        "audio.wav",
        AudioStemFormat::Wav,
        AudioCodec::PcmS16Le,
        AudioStemSource::Track {
            track_id: TrackId::new("trk_audio").unwrap(),
        },
    );
    let mut project = project_with(value);
    let mut audio_track = project.project.sequences[0].tracks[0].clone();
    audio_track.id = TrackId::new("trk_audio").unwrap();
    audio_track.kind = TrackKind::Audio;
    audio_track.order = 5;
    audio_track.clips.clear();
    project.project.sequences[0].tracks.push(audio_track);
    validate(&project).unwrap();

    let value = stem(
        "bus.wav",
        AudioStemFormat::Wav,
        AudioCodec::PcmS16Le,
        AudioStemSource::Bus {
            bus_id: BusId::new("bus_dialogue").unwrap(),
        },
    );
    let mut project = project_with(value);
    project.project.sequences[0].tracks[0].routing = TrackRouting::AudioBus {
        bus_id: BusId::new("bus_dialogue").unwrap(),
    };
    validate(&project).unwrap();
}

#[test]
fn audio_stems_reject_extension_codec_and_audio_parameter_mismatches() {
    let mut zero_rate = stem(
        "zero.wav",
        AudioStemFormat::Wav,
        AudioCodec::PcmS16Le,
        AudioStemSource::Master,
    );
    let DeliverableKind::AudioStem(settings) = &mut zero_rate.kind else {
        panic!()
    };
    settings.audio.sample_rate = 0;
    let mut zero_channels = zero_rate.clone();
    let DeliverableKind::AudioStem(settings) = &mut zero_channels.kind else {
        panic!()
    };
    settings.audio.sample_rate = 48_000;
    settings.audio.channels = 0;
    let mut many_channels = zero_channels.clone();
    let DeliverableKind::AudioStem(settings) = &mut many_channels.kind else {
        panic!()
    };
    settings.audio.channels = 33;

    for value in [
        stem(
            "wrong.flac",
            AudioStemFormat::Wav,
            AudioCodec::PcmS16Le,
            AudioStemSource::Master,
        ),
        stem(
            "wrong.wav",
            AudioStemFormat::Wav,
            AudioCodec::Flac,
            AudioStemSource::Master,
        ),
        stem(
            "wrong.flac",
            AudioStemFormat::Flac,
            AudioCodec::PcmS16Le,
            AudioStemSource::Master,
        ),
        zero_rate,
        zero_channels,
        many_channels,
    ] {
        assert_invalid(value, "OUTPUT_AUDIO_STEM");
    }
}

#[test]
fn stem_track_and_bus_sources_must_resolve_with_the_right_kind() {
    for track in ["trk_captions", "trk_missing"] {
        assert_invalid(
            stem(
                "track.wav",
                AudioStemFormat::Wav,
                AudioCodec::PcmS16Le,
                AudioStemSource::Track {
                    track_id: TrackId::new(track).unwrap(),
                },
            ),
            "OUTPUT_STEM_TRACK_NOT_FOUND",
        );
    }
    assert_invalid(
        stem(
            "bus.wav",
            AudioStemFormat::Wav,
            AudioCodec::PcmS16Le,
            AudioStemSource::Bus {
                bus_id: BusId::new("bus_missing").unwrap(),
            },
        ),
        "OUTPUT_STEM_BUS",
    );

    let value = stem(
        "bus.wav",
        AudioStemFormat::Wav,
        AudioCodec::PcmS16Le,
        AudioStemSource::Bus {
            bus_id: BusId::new("bus_captions-only").unwrap(),
        },
    );
    let mut project = project_with(value);
    project.project.sequences[0].tracks[1].routing = TrackRouting::AudioBus {
        bus_id: BusId::new("bus_captions-only").unwrap(),
    };
    assert_code(&validation_codes(&project), "OUTPUT_STEM_BUS");
}
