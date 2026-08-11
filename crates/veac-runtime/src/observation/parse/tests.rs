use super::*;

#[test]
fn parses_selected_pts_and_decode_progress() {
    let stderr = b"[Parsed_showinfo_0] n: 0 pts: 3 pts_time:0.125\n";
    assert_eq!(
        frame_pts(stderr, Rational::new(1, 24).unwrap()).unwrap(),
        RationalTime::new(3, 24).unwrap()
    );
    let progress = b"frame=1\nout_time_us=41667\nprogress=continue\nframe=4\nout_time_us=166667\nprogress=end\n";
    assert_eq!(
        decode_progress(progress).unwrap(),
        (4, Some(RationalTime::new(166667, 1_000_000).unwrap()))
    );
}

#[test]
fn selected_pts_uses_the_first_frame_that_reaches_the_filter() {
    let stderr = b"[Parsed_showinfo_0] n: 0 pts: 2 pts_time:0.2\n\
        [Parsed_showinfo_0] n: 1 pts: 7 pts_time:0.7\n";
    assert_eq!(
        frame_pts(stderr, Rational::new(1, 10).unwrap()).unwrap(),
        RationalTime::new(2, 10).unwrap()
    );
}

#[test]
fn rejects_missing_or_malformed_observation_metadata() {
    assert!(frame_pts(b"noise", Rational::new(1, 24).unwrap()).is_err());
    assert!(frame_pts(b"showinfo pts: bad", Rational::new(1, 24).unwrap()).is_err());
    assert!(decode_progress(b"progress=end\n").is_err());
    assert!(decode_progress(&[0xff]).is_err());
}

#[test]
fn rejects_selected_pts_that_overflow_the_time_base() {
    let stderr = b"[Parsed_showinfo_0] n: 0 pts: 9223372036854775807 pts_time:0\n";
    let error = frame_pts(stderr, Rational::new(2, 1).unwrap()).unwrap_err();
    assert_eq!(error.kind, crate::RuntimeErrorKind::ResourceLimit);
    assert_eq!(error.message, "selected frame PTS overflowed");
}

#[test]
fn rejects_selected_pts_outside_the_exact_time_domain() {
    let stderr = b"[Parsed_showinfo_0] n: 0 pts: 9007199254740992 pts_time:0\n";
    let error = frame_pts(stderr, Rational::new(1, 1).unwrap()).unwrap_err();
    assert_eq!(error.kind, crate::RuntimeErrorKind::General);
    assert_eq!(
        error.message,
        "selected frame PTS is outside the exact time domain"
    );
}

#[test]
fn rejects_decode_progress_outside_the_exact_time_domain() {
    let progress = b"frame=1\nout_time_us=9007199254740992\nprogress=end\n";
    let error = decode_progress(progress).unwrap_err();
    assert_eq!(error.kind, crate::RuntimeErrorKind::General);
    assert_eq!(error.message, "FFmpeg decode progress time is invalid");
}
