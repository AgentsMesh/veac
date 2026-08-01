use veac_plan::canonical::{HlsAudioEncoding, HlsPackage, RationalTime};

use super::super::{hls_encoding, output_video, time};
use super::{ENTRYPOINT, SEGMENT_PATTERN};

pub(super) fn build(settings: &HlsPackage, duration: RationalTime) -> Vec<String> {
    let mut args = Vec::new();
    for (index, rendition) in settings.renditions.iter().enumerate() {
        args.extend(output_video::stream_arguments(
            &hls_encoding::video(&rendition.encoding),
            index,
        ));
        args.extend([
            format!("-force_key_frames:v:{index}"),
            format!(
                "expr:gte(t,n_forced*{})",
                time::seconds(settings.segment_duration)
            ),
            format!("-sc_threshold:v:{index}"),
            "0".into(),
        ]);
        if let Some(audio) = &settings.audio {
            let HlsAudioEncoding::Aac(encoding) = &audio.encoding;
            args.extend([
                format!("-c:a:{index}"),
                "aac".into(),
                format!("-b:a:{index}"),
                encoding.bitrate_bps.to_string(),
                format!("-ar:a:{index}"),
                encoding.sample_rate_hz.to_string(),
                format!("-ac:a:{index}"),
                encoding.channel_layout.count().to_string(),
            ]);
        }
    }
    args.extend([
        "-f".into(),
        "hls".into(),
        "-hls_segment_type".into(),
        "mpegts".into(),
        "-hls_time".into(),
        time::seconds(settings.segment_duration),
        "-hls_list_size".into(),
        "0".into(),
        "-hls_playlist_type".into(),
        "vod".into(),
        "-hls_flags".into(),
        "independent_segments".into(),
        "-start_number".into(),
        "0".into(),
        "-var_stream_map".into(),
        stream_map(settings),
        "-master_pl_name".into(),
        ENTRYPOINT.into(),
        "-hls_segment_filename".into(),
        SEGMENT_PATTERN.into(),
        "-t".into(),
        time::seconds(duration),
    ]);
    args
}

fn stream_map(settings: &HlsPackage) -> String {
    settings
        .renditions
        .iter()
        .enumerate()
        .map(|(index, rendition)| {
            if settings.audio.is_some() {
                format!("v:{index},a:{index},name:{}", rendition.id.as_str())
            } else {
                format!("v:{index},name:{}", rendition.id.as_str())
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
