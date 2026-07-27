use veac_plan::canonical::{
    AlphaMode, HardwareBackend, HardwareSelection, PixelFormat, VideoCodec, VideoOutput,
    VideoProfile, VideoRateControl,
};

pub(super) fn arguments(video: &VideoOutput, hardware: HardwareSelection) -> Vec<String> {
    let mut args = vec![
        "-c:v".to_owned(),
        encoder(video.codec, hardware).to_owned(),
        "-pix_fmt".to_owned(),
        pixel_format(video.pixel_format).to_owned(),
    ];
    rate_control(&mut args, video);
    optional_pair(&mut args, "-g", video.gop_size);
    optional_pair(&mut args, "-bf", video.b_frames);
    if let Some(profile) = video.profile {
        args.extend(["-profile:v".to_owned(), profile_name(profile).to_owned()]);
    }
    if let Some(level) = &video.level {
        args.extend(["-level:v".to_owned(), level.clone()]);
    }
    if video.alpha == AlphaMode::Straight {
        args.extend(["-alpha_bits".to_owned(), "16".to_owned()]);
    }
    args.extend(super::color_output::arguments(video.color_space));
    args
}

fn rate_control(args: &mut Vec<String>, video: &VideoOutput) {
    match video.rate_control {
        VideoRateControl::Crf { value } => {
            args.extend(["-crf".to_owned(), value.to_string()]);
            if matches!(video.codec, VideoCodec::Vp9 | VideoCodec::Av1) {
                args.extend(["-b:v".to_owned(), "0".to_owned()]);
            }
        }
        VideoRateControl::Bitrate {
            target_bps,
            max_bps,
            buffer_bps,
        } => {
            args.extend(["-b:v".to_owned(), target_bps.to_string()]);
            optional_pair(args, "-maxrate", max_bps);
            optional_pair(args, "-bufsize", buffer_bps);
        }
        VideoRateControl::Lossless => match video.codec {
            VideoCodec::H264 | VideoCodec::H265 => {
                args.extend(["-crf".to_owned(), "0".to_owned()]);
            }
            VideoCodec::Vp9 => args.extend([
                "-lossless".to_owned(),
                "1".to_owned(),
                "-b:v".to_owned(),
                "0".to_owned(),
            ]),
            VideoCodec::Av1 => args.extend([
                "-crf".to_owned(),
                "0".to_owned(),
                "-b:v".to_owned(),
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
