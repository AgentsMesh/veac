use veac_ir::AudioCodec;

use super::codec;

#[test]
fn every_authored_audio_codec_has_an_exact_probe_name() {
    let cases = [
        (AudioCodec::Aac, "aac"),
        (AudioCodec::Opus, "opus"),
        (AudioCodec::Flac, "flac"),
        (AudioCodec::PcmS16Le, "pcm_s16le"),
        (AudioCodec::PcmS24Le, "pcm_s24le"),
        (AudioCodec::PcmS32Le, "pcm_s32le"),
    ];
    for (authored, observed) in cases {
        assert_eq!(codec(authored), observed);
    }
}
