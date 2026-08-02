use crate::VideoCodec;

pub(super) fn valid(codec: VideoCodec, value: &str) -> bool {
    syntax_valid(value)
        && match codec {
            VideoCodec::H264 => matches!(
                value,
                "1" | "1.1"
                    | "1.2"
                    | "1.3"
                    | "2"
                    | "2.1"
                    | "2.2"
                    | "3"
                    | "3.1"
                    | "3.2"
                    | "4"
                    | "4.1"
                    | "4.2"
                    | "5"
                    | "5.1"
                    | "5.2"
                    | "6"
                    | "6.1"
                    | "6.2"
            ),
            VideoCodec::H265 => matches!(
                value,
                "1" | "2"
                    | "2.1"
                    | "3"
                    | "3.1"
                    | "4"
                    | "4.1"
                    | "5"
                    | "5.1"
                    | "5.2"
                    | "6"
                    | "6.1"
                    | "6.2"
            ),
            VideoCodec::Vp9 => matches!(
                value,
                "1" | "1.1"
                    | "2"
                    | "2.1"
                    | "3"
                    | "3.1"
                    | "4"
                    | "4.1"
                    | "5"
                    | "5.1"
                    | "5.2"
                    | "6"
                    | "6.1"
                    | "6.2"
            ),
            VideoCodec::Av1 => av1(value),
            VideoCodec::ProRes | VideoCodec::DnxHr => false,
        }
}

fn av1(value: &str) -> bool {
    matches!(
        value,
        "2" | "2.1"
            | "2.2"
            | "2.3"
            | "3"
            | "3.1"
            | "3.2"
            | "3.3"
            | "4"
            | "4.1"
            | "4.2"
            | "4.3"
            | "5"
            | "5.1"
            | "5.2"
            | "5.3"
            | "6"
            | "6.1"
            | "6.2"
            | "6.3"
            | "7"
            | "7.1"
            | "7.2"
            | "7.3"
    )
}

fn syntax_valid(value: &str) -> bool {
    if value.is_empty() || value.len() > 8 {
        return false;
    }
    let mut parts = value.split('.');
    let valid_part =
        |part: &str| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit());
    parts.next().is_some_and(valid_part)
        && parts.next().is_none_or(valid_part)
        && parts.next().is_none()
}
