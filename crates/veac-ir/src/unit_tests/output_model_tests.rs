use crate::{test_support::sample_project, *};

#[test]
fn video_deliverable_accessors_return_none_without_a_video() {
    let mut output = sample_project().project.render_configs.remove(0);
    output.deliverables = vec![Deliverable {
        id: DeliverableId::new("dlv_frames").unwrap(),
        file_name: "frame-%d.png".to_owned(),
        kind: DeliverableKind::ImageSequence(ImageSequenceOutput {
            format: ImageFormat::Png,
            start_number: 1,
        }),
    }];

    assert_eq!(output.video_deliverable(), None);
    assert_eq!(output.video_deliverable_mut(), None);
}

#[test]
fn video_deliverable_accessors_find_and_mutate_a_non_first_video() {
    let mut output = sample_project().project.render_configs.remove(0);
    output.deliverables.insert(
        0,
        Deliverable {
            id: DeliverableId::new("dlv_captions").unwrap(),
            file_name: "captions.srt".to_owned(),
            kind: DeliverableKind::CaptionSidecar(CaptionSidecarOutput {
                format: CaptionSidecarFormat::Srt,
                track_ids: vec![TrackId::new("trk_captions").unwrap()],
            }),
        },
    );

    assert_eq!(
        output.video_deliverable().unwrap().container,
        OutputFormat::Mp4
    );
    output.video_deliverable_mut().unwrap().container = OutputFormat::Mov;
    let DeliverableKind::Video(video) = &output.deliverables[1].kind else {
        panic!("second deliverable must remain the video")
    };
    assert_eq!(video.container, OutputFormat::Mov);
    assert!(matches!(
        output.deliverables[0].kind,
        DeliverableKind::CaptionSidecar(_)
    ));
}

#[test]
fn image_sequence_patterns_parse_match_and_format_printf_widths() {
    let plain = ImageSequencePattern::parse("frame-%d.png").unwrap();
    assert!(plain.matches("frame-12.png"));
    assert_eq!(plain.format_index(7), "frame-7.png");

    let padded = ImageSequencePattern::parse("frame-%04d.png").unwrap();
    assert!(!padded.matches("frame-7.png"));
    assert!(padded.matches("frame-0007.png"));
    assert!(padded.matches("frame-10000.png"));
    assert_eq!(padded.format_index(7), "frame-0007.png");
    assert!(plain.overlaps(padded));
    assert!(!padded.overlaps(ImageSequencePattern::parse("other-%04d.png").unwrap()));

    for invalid in [
        "frame.png",
        "frame-%0d.png",
        "frame-%00d.png",
        "frame-%d-%d.png",
        "frame-%0999999999d.png",
        "nested/frame-%04d.png",
    ] {
        assert!(ImageSequencePattern::parse(invalid).is_none(), "{invalid}");
    }
    let oversized_generated_name = format!("{}%d.png", "a".repeat(232));
    assert!(ImageSequencePattern::parse(&oversized_generated_name).is_none());
}
