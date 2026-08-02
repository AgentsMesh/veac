use crate::{test_support::sample_project, *};

#[test]
fn every_dnxhr_profile_has_one_exact_pixel_contract() {
    for (profile, pixel) in [
        (VideoProfile::DnxHrLb, PixelFormat::Yuv422p),
        (VideoProfile::DnxHrSq, PixelFormat::Yuv422p),
        (VideoProfile::DnxHrHq, PixelFormat::Yuv422p),
        (VideoProfile::DnxHrHqx, PixelFormat::Yuv422p10le),
        (VideoProfile::DnxHr444, PixelFormat::Yuv444p10le),
    ] {
        let project = project(profile, pixel);
        validate(&project).unwrap_or_else(|error| panic!("{profile:?}/{pixel:?}: {error}"));
    }
}

#[test]
fn crossed_dnxhr_profiles_and_pixels_fail_closed() {
    for (profile, pixel) in [
        (VideoProfile::DnxHrHq, PixelFormat::Yuv422p10le),
        (VideoProfile::DnxHrHqx, PixelFormat::Yuv422p),
        (VideoProfile::DnxHr444, PixelFormat::Yuv422p10le),
    ] {
        assert!(has_code(&project(profile, pixel), "OUTPUT_VIDEO_SETTINGS"));
    }
}

#[test]
fn mxf_geometry_uses_the_probed_ffmpeg_boundary() {
    for rate in [
        Rational::new(24, 1).unwrap(),
        Rational::new(25, 1).unwrap(),
        Rational::new(30, 1).unwrap(),
        Rational::new(50, 1).unwrap(),
        Rational::new(60, 1).unwrap(),
        Rational::new(24_000, 1_001).unwrap(),
        Rational::new(30_000, 1_001).unwrap(),
        Rational::new(60_000, 1_001).unwrap(),
    ] {
        let mut project = project(VideoProfile::DnxHrHq, PixelFormat::Yuv422p);
        let output = &mut project.project.render_configs[0];
        let raster = output.raster.as_mut().unwrap();
        (raster.width, raster.height, raster.frame_rate) = (256, 120, rate);
        validate(&project).unwrap();
    }

    let mut too_small = project(VideoProfile::DnxHrHq, PixelFormat::Yuv422p);
    let raster = too_small.project.render_configs[0].raster.as_mut().unwrap();
    (raster.width, raster.height) = (96, 54);
    assert!(has_code(&too_small, "OUTPUT_MXF_SETTINGS"));

    let mut bad_rate = project(VideoProfile::DnxHrHq, PixelFormat::Yuv422p);
    bad_rate.project.render_configs[0]
        .raster
        .as_mut()
        .unwrap()
        .frame_rate = Rational::new(10, 1).unwrap();
    assert!(has_code(&bad_rate, "OUTPUT_MXF_SETTINGS"));
}

#[test]
fn mxf_audio_accepts_only_48khz_pcm16_or_pcm24() {
    let mut pcm32 = project(VideoProfile::DnxHrHq, PixelFormat::Yuv422p);
    audio(&mut pcm32).codec = AudioCodec::PcmS32Le;
    assert!(has_code(&pcm32, "OUTPUT_AUDIO"));

    for rate in [44_100, 96_000] {
        let mut project = project(VideoProfile::DnxHrHq, PixelFormat::Yuv422p);
        audio(&mut project).sample_rate = rate;
        assert!(has_code(&project, "OUTPUT_VIDEO_SETTINGS"));
    }
}

pub(crate) fn project(profile: VideoProfile, pixel: PixelFormat) -> ProjectEnvelope {
    let mut project = sample_project();
    let output = &mut project.project.render_configs[0];
    output.deliverables[0].target = DeliverableTarget::File {
        name: "main.mxf".to_owned(),
    };
    let id = DeliverableId::new("dlv_main").unwrap();
    let video = output.video_deliverable_mut(&id).unwrap();
    video.container = OutputFormat::Mxf;
    video.video = VideoOutput {
        codec: VideoCodec::DnxHr,
        pixel_format: pixel,
        alpha: AlphaMode::Opaque,
        color_space: None,
        rate_control: VideoRateControl::Lossless,
        gop_size: None,
        b_frames: None,
        profile: Some(profile),
        level: None,
    };
    video.audio = Some(AudioOutput {
        codec: AudioCodec::PcmS24Le,
        sample_rate: 48_000,
        channels: 2,
    });
    project
}

fn audio(project: &mut ProjectEnvelope) -> &mut AudioOutput {
    project.project.render_configs[0]
        .video_deliverable_mut(&DeliverableId::new("dlv_main").unwrap())
        .unwrap()
        .audio
        .as_mut()
        .unwrap()
}

fn has_code(project: &ProjectEnvelope, code: &str) -> bool {
    validate(project)
        .unwrap_err()
        .diagnostics()
        .iter()
        .any(|value| value.code == code)
}
