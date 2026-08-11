use veac_project::*;

pub fn operations() -> Vec<MediaDerivation> {
    vec![
        MediaDerivation::ProxyVideo {
            source: source(),
            source_stream: video_stream(),
            source_clock: identity(duration()),
            width: 16,
            height: 16,
            frame_rate: ProjectRational::new(10, 1),
            crf: 28,
        },
        MediaDerivation::ProxyAudio {
            source: source(),
            source_stream: audio_stream(),
            source_clock: identity(duration()),
            sample_rate: 48000,
            channels: 2,
        },
        MediaDerivation::Thumbnail {
            source: source(),
            source_stream: video_stream(),
            at: ProjectRational::new(1, 10),
            width: 16,
            height: 16,
        },
        MediaDerivation::Waveform {
            source: source(),
            source_stream: audio_stream(),
            source_clock: identity(duration()),
            sample_rate: 48000,
            width: 64,
            height: 16,
            color: "#00ff00".into(),
        },
        MediaDerivation::OpticalFlow {
            source: source(),
            source_stream: video_stream(),
            source_clock: identity(duration()),
            width: 32,
            height: 32,
            frame_rate: ProjectRational::new(20, 1),
            method: ProjectOpticalFlowMethod::MotionCompensated,
        },
        MediaDerivation::SourceSegment {
            source: source(),
            video_stream: video_stream(),
            start: ProjectRational::new(0, 1),
            duration: ProjectRational::new(1, 5),
            width: 16,
            height: 16,
            frame_rate: ProjectRational::new(10, 1),
            audio: Some(ProjectSegmentAudio {
                source_stream: audio_stream(),
                sample_rate: 48000,
                channels: 2,
            }),
            crf: 23,
        },
    ]
}

pub fn configure(target: &mut ProjectTarget, operation: MediaDerivation) {
    let media_type = operation.output_media_type();
    target.entry = ProjectTargetEntry::MediaDerivation { operation };
    target.inputs = vec![ProjectInput {
        id: source(),
        source: ProjectInputSource::ProjectMaterial {
            path: ProjectPath::new("source.mp4"),
        },
    }];
    target.outputs = vec![ProjectOutput::Media {
        id: OutputId::from("video"),
        media_type,
    }];
}

fn source() -> InputId {
    InputId::from("source")
}
fn duration() -> ProjectRational {
    ProjectRational::new(2, 5)
}
fn identity(duration: ProjectRational) -> ProjectSourceClock {
    ProjectSourceClock::Identity { duration }
}
fn video_stream() -> ProjectStreamSelection {
    ProjectStreamSelection {
        global_index: 0,
        type_index: 0,
    }
}
fn audio_stream() -> ProjectStreamSelection {
    ProjectStreamSelection {
        global_index: 1,
        type_index: 0,
    }
}
