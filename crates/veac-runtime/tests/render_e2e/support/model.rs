use super::*;

const TIMEBASE: u32 = 1_000;
pub(crate) const WIDTH: u32 = 96;
pub(crate) const HEIGHT: u32 = 54;
pub(crate) const FPS: i64 = 10;

pub(crate) fn project(with_audio: bool) -> ProjectEnvelope {
    let mut value: ProjectEnvelope = serde_json::from_str(include_str!(
        "../../../../veac-ir/tests/fixtures/minimal-project.json"
    ))
    .expect("canonical fixture");
    value.project.timebase = TIMEBASE;
    value.project.materials.clear();
    value.project.sequences = vec![sequence("seq_main", Vec::new())];
    let output = &mut value.project.render_configs[0];
    let raster = output.raster.as_mut().expect("raster fixture");
    raster.width = WIDTH;
    raster.height = HEIGHT;
    raster.frame_rate = ratio(FPS, 1);
    let deliverable_id = output.deliverables[0].id.clone();
    output.video_deliverable_mut(&deliverable_id).unwrap().audio =
        with_audio.then_some(AudioOutput {
            codec: AudioCodec::Aac,
            sample_rate: 48_000,
            channels: 1,
        });
    value
}

pub(crate) fn sequence(id: &str, tracks: Vec<Track>) -> Sequence {
    Sequence {
        id: SequenceId::new(id).unwrap(),
        name: id.to_owned(),
        settings: SequenceSettings {
            width: WIDTH,
            height: HEIGHT,
            frame_rate: ratio(FPS, 1),
            sample_rate: 48_000,
        },
        tracks,
        applies: Vec::new(),
        metadata: BTreeMap::new(),
    }
}

pub(crate) fn track(id: &str, kind: TrackKind, order: i32, clips: Vec<Clip>) -> Track {
    Track {
        id: TrackId::new(id).unwrap(),
        kind,
        order,
        placement_mode: PlacementMode::Free,
        state: TrackState {
            enabled: true,
            muted: false,
            solo: false,
            locked: false,
        },
        routing: TrackRouting::Default,
        clips,
    }
}

pub(crate) fn media_clip(id: &str, material: &str, start_ms: i64, duration_ms: i64) -> Clip {
    let mut clip = clip(
        id,
        start_ms,
        duration_ms,
        ClipSource::Media {
            material_id: MaterialId::new(material).unwrap(),
        },
    );
    clip.source_mapping = Some(SourceMapping::linear(time(0), ratio(1, 1)));
    clip
}

pub(crate) fn solid_clip(id: &str, color: Color, start_ms: i64, duration_ms: i64) -> Clip {
    clip(
        id,
        start_ms,
        duration_ms,
        ClipSource::Generated {
            generator: Generator::Solid { color },
        },
    )
}

pub(crate) fn nested_clip(id: &str, sequence_id: &str, start_ms: i64, duration_ms: i64) -> Clip {
    let mut clip = clip(
        id,
        start_ms,
        duration_ms,
        ClipSource::Sequence {
            sequence_id: SequenceId::new(sequence_id).unwrap(),
        },
    );
    clip.source_mapping = Some(SourceMapping::linear(time(0), ratio(1, 1)));
    clip
}

pub(crate) fn text_clip(
    id: &str,
    text: &str,
    style: TextStyle,
    start_ms: i64,
    duration_ms: i64,
) -> Clip {
    clip(
        id,
        start_ms,
        duration_ms,
        ClipSource::Text {
            text: text.to_owned(),
            style,
        },
    )
}

fn clip(id: &str, start_ms: i64, duration_ms: i64, source: ClipSource) -> Clip {
    Clip {
        id: ItemId::new(id).unwrap(),
        enabled: true,
        record_range: TimeRange::new(time(start_ms), time(duration_ms)).unwrap(),
        source,
        source_mapping: None,
        visual: None,
        audio: None,
        effects: Vec::new(),
        replaceable: None,
        template_editable_text: false,
        metadata: BTreeMap::new(),
    }
}

pub(crate) fn audio_properties(gain: f64) -> AudioProperties {
    AudioProperties {
        gain: Animatable::constant(gain),
        pan: Animatable::constant(0.0),
        muted: false,
        normalize: false,
        pitch_policy: PitchPolicy::Preserve,
        processors: vec![],
        crossfade: None,
    }
}

pub(crate) fn linear_mapping(
    repeat: u32,
    rate: Rational,
    frame_synthesis: FrameSynthesisPolicy,
) -> SourceMapping {
    SourceMapping {
        time_map: SourceTimeMap::Linear {
            source_start: time(0),
            rate,
            repeat,
            direction: PlaybackDirection::Forward,
        },
        frame_synthesis,
        out_of_range: SourceOutOfRangePolicy::Strict,
    }
}

pub(crate) fn color(red: u8, green: u8, blue: u8) -> Color {
    Color {
        red,
        green,
        blue,
        alpha: 255,
    }
}

pub(crate) fn time(milliseconds: i64) -> RationalTime {
    RationalTime::new(milliseconds, TIMEBASE).unwrap()
}

pub(crate) fn ratio(numerator: i64, denominator: u32) -> Rational {
    Rational::new(numerator, denominator).unwrap()
}
