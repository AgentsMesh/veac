use super::{graph::Graph, rgb_planes};

#[test]
fn drops_alpha_through_explicit_rgb_planes() {
    let mut graph = Graph::default();
    let output = rgb_planes::without_alpha(&mut graph, "input", "rgbv");
    let rendered = graph.render_with_inputs(&["input".to_owned()], &[]);
    for marker in [
        "format=gbrap16le",
        "mergeplanes=format=gbrp16le",
        "map0s=0:map0p=0",
        "map1s=0:map1p=1",
        "map2s=0:map2p=2",
    ] {
        assert!(rendered.contains(marker), "missing {marker}: {rendered}");
    }
    assert!(!rendered.contains("extractplanes"));
    assert!(rendered.ends_with(&format!("[{output}]")), "{rendered}");
    assert!(!rendered.contains("]format=gbrp16le"), "{rendered}");
}
