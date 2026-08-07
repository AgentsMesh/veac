use veac_plan::canonical::{
    AlphaMode, HardwareBackend, HardwareSelection, PixelFormat, VideoCodec, VideoOutput,
    VideoProfile, VideoRateControl,
};

pub(super) fn arguments(video: &VideoOutput, hardware: HardwareSelection) -> Vec<String> {
    arguments_with_suffix(video, hardware, "")
}

pub(super) fn stream_arguments(video: &VideoOutput, stream: usize) -> Vec<String> {
    arguments_with_suffix(video, HardwareSelection::Software, &format!(":{stream}"))
}

fn arguments_with_suffix(
    video: &VideoOutput,
    hardware: HardwareSelection,
    suffix: &str,
) -> Vec<String> {
    let mut args = vec![
        option("-c:v", "-c:v", suffix),
        encoder(video.codec, hardware).to_owned(),
        option("-pix_fmt", "-pix_fmt:v", suffix),
        pixel_format(video.pixel_format).to_owned(),
    ];
    rate_control(&mut args, video, suffix);
    optional_pair(&mut args, &option("-g", "-g:v", suffix), video.gop_size);
    optional_pair(&mut args, &option("-bf", "-bf:v", suffix), video.b_frames);
    if let Some(profile) = video.profile {
        args.extend([
            option("-profile:v", "-profile:v", suffix),
            profile_name(profile).to_owned(),
        ]);
    }
    if let Some(level) = &video.level {
        args.extend([
            option("-level:v", "-level:v", suffix),
            level_argument(video.codec, level),
        ]);
    }
    if video.alpha == AlphaMode::Straight {
        args.extend([
            option("-alpha_bits", "-alpha_bits:v", suffix),
            "16".to_owned(),
        ]);
    }
    args.extend(if suffix.is_empty() {
        super::color_output::arguments(video.color_space)
    } else {
        super::color_output::stream_arguments(video.color_space, suffix)
    });
    args
}

fn level_argument(codec: VideoCodec, value: &str) -> String {
    if codec != VideoCodec::Av1 {
        return value.to_owned();
    }
    let (major, minor) = value
        .split_once('.')
        .map_or((value, "0"), |(major, minor)| (major, minor));
    let major = major.parse::<u8>().expect("validated AV1 level major");
    let minor = minor.parse::<u8>().expect("validated AV1 level minor");
    ((major - 2) * 4 + minor).to_string()
}

fn rate_control(args: &mut Vec<String>, video: &VideoOutput, suffix: &str) {
    match video.rate_control {
        VideoRateControl::Crf { value } => {
            args.extend([option("-crf", "-crf:v", suffix), value.to_string()]);
            if matches!(video.codec, VideoCodec::Vp9 | VideoCodec::Av1) {
                args.extend([option("-b:v", "-b:v", suffix), "0".to_owned()]);
            }
        }
        VideoRateControl::Bitrate {
            target_bps,
            max_bps,
            buffer_size_bits,
        } => {
            args.extend([option("-b:v", "-b:v", suffix), target_bps.to_string()]);
            optional_pair(args, &option("-maxrate", "-maxrate:v", suffix), max_bps);
            optional_pair(
                args,
                &option("-bufsize", "-bufsize:v", suffix),
                buffer_size_bits,
            );
        }
        VideoRateControl::Lossless => match video.codec {
            VideoCodec::H264 | VideoCodec::H265 => {
                args.extend([option("-crf", "-crf:v", suffix), "0".to_owned()]);
            }
            VideoCodec::Vp9 => args.extend([
                option("-lossless", "-lossless:v", suffix),
                "1".to_owned(),
                option("-b:v", "-b:v", suffix),
                "0".to_owned(),
            ]),
            VideoCodec::Av1 => args.extend([
                option("-crf", "-crf:v", suffix),
                "0".to_owned(),
                option("-b:v", "-b:v", suffix),
                "0".to_owned(),
            ]),
            VideoCodec::ProRes | VideoCodec::DnxHr => {}
        },
    }
}

fn optional_pair<T: ToString>(args: &mut Vec<String>, name: &str, value: Option<T>) {
    if let Some(value) = value {
        args.extend([name.to_owned(), value.to_string()]);
    }
}

fn option(unscoped: &str, scoped: &str, suffix: &str) -> String {
    if suffix.is_empty() {
        unscoped.to_owned()
    } else {
        format!("{scoped}{suffix}")
    }
}

pub(super) fn encoder(codec: VideoCodec, hardware: HardwareSelection) -> &'static str {
    let HardwareSelection::Explicit { backend } = hardware else {
        return software_encoder(codec);
    };
    match (codec, backend) {
        (VideoCodec::H264, HardwareBackend::VideoToolbox) => "h264_videotoolbox",
        (VideoCodec::H265, HardwareBackend::VideoToolbox) => "hevc_videotoolbox",
        (VideoCodec::H264, HardwareBackend::Nvenc) => "h264_nvenc",
        (VideoCodec::H265, HardwareBackend::Nvenc) => "hevc_nvenc",
        (VideoCodec::Av1, HardwareBackend::Nvenc) => "av1_nvenc",
        (VideoCodec::H264, HardwareBackend::Qsv) => "h264_qsv",
        (VideoCodec::H265, HardwareBackend::Qsv) => "hevc_qsv",
        (VideoCodec::Vp9, HardwareBackend::Qsv) => "vp9_qsv",
        (VideoCodec::Av1, HardwareBackend::Qsv) => "av1_qsv",
        (VideoCodec::H264, HardwareBackend::Vaapi) => "h264_vaapi",
        (VideoCodec::H265, HardwareBackend::Vaapi) => "hevc_vaapi",
        (VideoCodec::Vp9, HardwareBackend::Vaapi) => "vp9_vaapi",
        (VideoCodec::Av1, HardwareBackend::Vaapi) => "av1_vaapi",
        _ => software_encoder(codec),
    }
}

fn software_encoder(codec: VideoCodec) -> &'static str {
    match codec {
        VideoCodec::H264 => "libx264",
        VideoCodec::H265 => "libx265",
        VideoCodec::Vp9 => "libvpx-vp9",
        VideoCodec::Av1 => "libaom-av1",
        VideoCodec::ProRes => "prores_ks",
        VideoCodec::DnxHr => "dnxhd",
    }
}

fn pixel_format(format: PixelFormat) -> &'static str {
    match format {
        PixelFormat::Yuv420p => "yuv420p",
        PixelFormat::Yuv420p10le => "yuv420p10le",
        PixelFormat::Yuv422p => "yuv422p",
        PixelFormat::Yuv422p10le => "yuv422p10le",
        PixelFormat::Yuv444p10le => "yuv444p10le",
        PixelFormat::Yuva444p10le => "yuva444p10le",
    }
}

fn profile_name(profile: VideoProfile) -> &'static str {
    match profile {
        VideoProfile::H264Baseline => "baseline",
        VideoProfile::H264Main | VideoProfile::H265Main => "main",
        VideoProfile::H264High => "high",
        VideoProfile::H264High10 => "high10",
        VideoProfile::H265Main10 => "main10",
        VideoProfile::Vp9Profile0 | VideoProfile::Av1Main => "0",
        VideoProfile::Vp9Profile2 => "2",
        VideoProfile::ProRes4444 => "4",
        VideoProfile::DnxHrLb => "dnxhr_lb",
        VideoProfile::DnxHrSq => "dnxhr_sq",
        VideoProfile::DnxHrHq => "dnxhr_hq",
        VideoProfile::DnxHrHqx => "dnxhr_hqx",
        VideoProfile::DnxHr444 => "dnxhr_444",
    }
}
