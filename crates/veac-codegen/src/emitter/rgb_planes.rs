use super::graph::Graph;

pub(super) fn without_alpha(graph: &mut Graph, input: &str, prefix: &str) -> String {
    let prepared = graph.filter(&[input], "format=gbrap16le", "rp");
    graph.filter(
        &[&prepared],
        concat!(
            "mergeplanes=format=gbrp16le:",
            "map0s=0:map0p=0:map1s=0:map1p=1:map2s=0:map2p=2"
        ),
        prefix,
    )
}
