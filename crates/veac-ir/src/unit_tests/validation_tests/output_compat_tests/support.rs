use crate::{
    ColorMatrix, ColorPrimaries, ColorRange, ColorSpace, ColorTransfer, VideoCodec,
    VideoDeliverable, VideoOutput, VideoRateControl,
};

pub(super) fn video_for(codec: VideoCodec) -> VideoOutput {
    let mut value = VideoOutput {
        codec,
        rate_control: if codec == VideoCodec::ProRes {
            VideoRateControl::Lossless
        } else {
            VideoRateControl::Crf { value: 23 }
        },
        ..VideoOutput::default()
    };
    if codec == VideoCodec::ProRes {
        value.pixel_format = crate::PixelFormat::Yuva444p10le;
        value.alpha = crate::AlphaMode::Straight;
        value.profile = Some(crate::VideoProfile::ProRes4444);
    }
    value
}

pub(super) fn delivery_for(codec: VideoCodec) -> VideoDeliverable {
    let mut value = VideoDeliverable {
        video: video_for(codec),
        ..VideoDeliverable::default()
    };
    if codec == VideoCodec::ProRes {
        value.container = crate::OutputFormat::Mov;
    }
    value
}

pub(super) fn color_space(
    primaries: ColorPrimaries,
    transfer: ColorTransfer,
    matrix: ColorMatrix,
) -> ColorSpace {
    ColorSpace {
        primaries,
        transfer,
        matrix,
        range: ColorRange::Limited,
    }
}
