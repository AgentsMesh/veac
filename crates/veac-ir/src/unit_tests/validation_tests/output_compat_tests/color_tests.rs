use crate::{
    video_delivery_valid, ColorMatrix, ColorPrimaries, ColorTransfer, PixelFormat, VideoCodec,
};

use super::support::{color_space, delivery_for};

#[test]
fn absent_or_standard_dynamic_range_color_is_unrestricted() {
    let mut value = delivery_for(VideoCodec::H264);
    assert!(video_delivery_valid(&value));

    value.video.color_space = Some(color_space(
        ColorPrimaries::Bt709,
        ColorTransfer::Bt709,
        ColorMatrix::Rgb,
    ));
    assert!(video_delivery_valid(&value));
}

#[test]
fn pq_and_hlg_bt2020_are_valid_for_ten_bit_h265_and_av1() {
    for transfer in [ColorTransfer::Smpte2084, ColorTransfer::AribStdB67] {
        for codec in [VideoCodec::H265, VideoCodec::Av1] {
            let mut value = delivery_for(codec);
            value.video.pixel_format = PixelFormat::Yuv420p10le;
            value.video.color_space = Some(color_space(
                ColorPrimaries::Bt2020,
                transfer,
                ColorMatrix::Bt2020Ncl,
            ));
            assert!(
                video_delivery_valid(&value),
                "rejected {transfer:?}/{codec:?}"
            );
        }
    }
}

#[test]
fn wide_gamut_non_hdr_uses_the_same_ten_bit_delivery_contract() {
    for codec in [VideoCodec::H265, VideoCodec::Av1] {
        let mut value = delivery_for(codec);
        value.video.pixel_format = PixelFormat::Yuv420p10le;
        value.video.color_space = Some(color_space(
            ColorPrimaries::Bt2020,
            ColorTransfer::Bt2020_10,
            ColorMatrix::Bt2020Ncl,
        ));
        assert!(video_delivery_valid(&value));
    }
}

#[test]
fn hdr_rejects_missing_wide_primaries_matrix_or_bit_depth() {
    let invalid = [
        (
            ColorPrimaries::Bt709,
            ColorMatrix::Bt2020Ncl,
            PixelFormat::Yuv420p10le,
        ),
        (
            ColorPrimaries::Bt2020,
            ColorMatrix::Bt709,
            PixelFormat::Yuv420p10le,
        ),
        (
            ColorPrimaries::Bt2020,
            ColorMatrix::Bt2020Ncl,
            PixelFormat::Yuv420p,
        ),
    ];

    for (primaries, matrix, pixel_format) in invalid {
        let mut value = delivery_for(VideoCodec::H265);
        value.video.pixel_format = pixel_format;
        value.video.color_space = Some(color_space(primaries, ColorTransfer::Smpte2084, matrix));
        assert!(!video_delivery_valid(&value));
    }
}

#[test]
fn hdr_rejects_codecs_without_the_delivery_contract() {
    for codec in [VideoCodec::H264, VideoCodec::Vp9, VideoCodec::ProRes] {
        let mut value = delivery_for(codec);
        value.video.pixel_format = PixelFormat::Yuv420p10le;
        value.video.color_space = Some(color_space(
            ColorPrimaries::Bt2020,
            ColorTransfer::AribStdB67,
            ColorMatrix::Bt2020Ncl,
        ));
        assert!(!video_delivery_valid(&value), "accepted HDR for {codec:?}");
    }
}
