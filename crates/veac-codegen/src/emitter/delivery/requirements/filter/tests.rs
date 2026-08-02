use super::*;

#[test]
fn parses_chains_labels_quotes_and_escaped_separators() {
    let graph = "[0:v]zscale=matrix=bt709,format=gbrp16le[a];[a]subtitles=filename='data\\:x\\,y',deshake[out]";
    assert_eq!(
        names(graph),
        BTreeSet::from([
            "deshake".to_owned(),
            "format".to_owned(),
            "subtitles".to_owned(),
            "zscale".to_owned(),
        ])
    );
}
