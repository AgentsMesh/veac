use std::collections::BTreeSet;

use super::capability::{
    parse_decoders, parse_demuxers, parse_encoders, parse_filters, parse_muxers, parse_names,
};

#[test]
fn parses_only_encoder_and_muxer_table_rows() {
    let encoders = "Encoders:\n V..... dnxhd DNxHD\n A..... pcm_s24le PCM\n ------\n";
    assert_eq!(
        parse_encoders(encoders),
        BTreeSet::from(["dnxhd".to_owned(), "pcm_s24le".to_owned()])
    );
    let muxers = "Muxers:\n E mov,mp4 QuickTime\n D mxf MXF\n --\n";
    assert_eq!(
        parse_muxers(muxers),
        BTreeSet::from(["mov".to_owned(), "mp4".to_owned()])
    );
}

#[test]
fn parses_decoder_demuxer_filter_and_hardware_tables() {
    let decoders = "Decoders:\n V....D h264 H.264\n ------\n";
    assert_eq!(parse_decoders(decoders), BTreeSet::from(["h264".into()]));
    let demuxers = "Formats:\n D mov,mp4 QuickTime\n E mxf MXF\n ---\n";
    assert_eq!(
        parse_demuxers(demuxers),
        BTreeSet::from(["mov".into(), "mp4".into()])
    );
    let filters = "Filters:\n TS zscale V->V scale\n XX rejected V->V\n";
    assert_eq!(parse_filters(filters), BTreeSet::from(["zscale".into()]));
    let hardware = "Hardware acceleration methods:\nvideotoolbox\nnot a name\n";
    assert_eq!(
        parse_names(hardware),
        BTreeSet::from(["videotoolbox".into()])
    );
}
