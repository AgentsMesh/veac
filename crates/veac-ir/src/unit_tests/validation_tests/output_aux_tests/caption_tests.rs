use super::*;

#[test]
fn caption_formats_and_sorted_caption_track_refs_are_valid() {
    assert_valid(caption(
        "captions.SRT",
        CaptionSidecarFormat::Srt,
        &["trk_captions"],
    ));

    let value = caption(
        "captions.VTT",
        CaptionSidecarFormat::WebVtt,
        &["trk_captions", "trk_subtitles"],
    );
    let mut project = project_with(value);
    let mut extra = project.project.sequences[0].tracks[1].clone();
    extra.id = TrackId::new("trk_subtitles").unwrap();
    extra.order = 11;
    extra.clips.clear();
    project.project.sequences[0].tracks.push(extra);
    validate(&project).unwrap();
}

#[test]
fn caption_sidecars_reject_wrong_format_empty_or_unsorted_tracks() {
    for value in [
        caption("captions.vtt", CaptionSidecarFormat::Srt, &["trk_captions"]),
        caption("captions.srt", CaptionSidecarFormat::Srt, &[]),
        caption(
            "captions.srt",
            CaptionSidecarFormat::Srt,
            &["trk_captions", "trk_captions"],
        ),
        caption(
            "captions.vtt",
            CaptionSidecarFormat::WebVtt,
            &["trk_zulu", "trk_alpha"],
        ),
    ] {
        assert_invalid(value, "OUTPUT_CAPTION_SIDECAR");
    }
}

#[test]
fn caption_refs_must_exist_on_the_selected_sequence_and_be_caption_tracks() {
    for track in ["trk_missing", "trk_video"] {
        assert_invalid(
            caption("captions.srt", CaptionSidecarFormat::Srt, &[track]),
            "OUTPUT_CAPTION_TRACK_NOT_FOUND",
        );
    }
}

#[test]
fn plain_caption_sidecars_require_exact_milliseconds() {
    for (file, format) in [
        ("captions.srt", CaptionSidecarFormat::Srt),
        ("captions.vtt", CaptionSidecarFormat::WebVtt),
    ] {
        let mut project = project_with(caption(file, format, &["trk_captions"]));
        project.project.sequences[0].tracks[1].clips[0]
            .record_range
            .start = RationalTime::new(1, 600).unwrap();
        assert_code(&validation_codes(&project), "OUTPUT_CAPTION_TIME");
    }
}
