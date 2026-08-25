use super::{PreparedSourceGraph, SourceAuthority};

type Node<'a> = (&'a str, &'a str, SourceAuthority);
type Route<'a> = (&'a str, &'a str, &'a str);

const NODES: [Node<'static>; 3] = [
    ("main.veac", "main\n", SourceAuthority::Project),
    (
        "packages/demo.veac",
        "module {}\n",
        SourceAuthority::ReadOnlyDependency,
    ),
    (
        "packages/leaf.veac",
        "module { export const int n = 1; }\n",
        SourceAuthority::ReadOnlyDependency,
    ),
];
const ROUTES: [Route<'static>; 2] = [
    ("main.veac", "package:demo", "packages/demo.veac"),
    ("packages/demo.veac", "./leaf.veac", "packages/leaf.veac"),
];

fn graph(root: &str, nodes: &[Node<'_>], routes: &[Route<'_>]) -> PreparedSourceGraph {
    PreparedSourceGraph::new(
        root.to_owned(),
        nodes
            .iter()
            .map(|(id, source, _)| ((*id).to_owned(), (*source).to_owned()))
            .collect(),
        nodes
            .iter()
            .map(|(id, _, authority)| ((*id).to_owned(), *authority))
            .collect(),
        routes
            .iter()
            .map(|(importer, requested, resolved)| {
                (
                    ((*importer).to_owned(), (*requested).to_owned()),
                    (*resolved).to_owned(),
                )
            })
            .collect(),
    )
}

fn fixture() -> PreparedSourceGraph {
    graph("main.veac", &NODES, &ROUTES)
}

#[test]
fn complete_revision_has_a_fixed_v1_vector() {
    assert_eq!(
        fixture().complete_revision().sha256(),
        "84539c0772d0605406be607e54542bd587a8e9a35560cd2952913ee7fd1cdbec"
    );
}

#[test]
fn complete_revision_uses_canonical_node_and_route_order() {
    let nodes = [NODES[2], NODES[0], NODES[1]];
    let routes = [ROUTES[1], ROUTES[0]];
    assert_eq!(
        graph("main.veac", &nodes, &routes).complete_revision(),
        fixture().complete_revision()
    );
}

#[test]
fn complete_revision_frames_node_and_route_fields() {
    let node_left = graph(
        "root",
        &[
            ("root", "", SourceAuthority::Project),
            ("a", "\0bc", SourceAuthority::Project),
        ],
        &[],
    );
    let node_right = graph(
        "root",
        &[
            ("root", "", SourceAuthority::Project),
            ("a\0", "bc", SourceAuthority::Project),
        ],
        &[],
    );
    let route_nodes = [
        ("root", "", SourceAuthority::Project),
        ("a", "", SourceAuthority::Project),
        ("ab", "", SourceAuthority::Project),
    ];
    let route_left = graph("root", &route_nodes, &[("a", "bc", "root")]);
    let route_right = graph("root", &route_nodes, &[("ab", "c", "root")]);
    assert_ne!(
        node_left.complete_revision(),
        node_right.complete_revision()
    );
    assert_ne!(
        route_left.complete_revision(),
        route_right.complete_revision()
    );
}

#[test]
fn complete_revision_binds_root_bytes_authority_and_routes() {
    let base = fixture().complete_revision();
    let changed_root = graph("packages/demo.veac", &NODES, &ROUTES);
    let mut bytes = NODES;
    bytes[0].1 = "changed\n";
    let changed_bytes = graph("main.veac", &bytes, &ROUTES);
    let mut authorities = NODES;
    authorities[0].2 = SourceAuthority::ReadOnlyDependency;
    let changed_authority = graph("main.veac", &authorities, &ROUTES);
    let mut routes = ROUTES;
    routes[0].1 = "package:renamed";
    let changed_route = graph("main.veac", &NODES, &routes);
    for changed in [
        changed_root,
        changed_bytes,
        changed_authority,
        changed_route,
    ] {
        assert_ne!(base, changed.complete_revision());
    }
}

#[test]
fn complete_revision_hashes_exact_utf8_without_normalization() {
    let composed = graph(
        "main.veac",
        &[("main.veac", "\u{e9}", SourceAuthority::Project)],
        &[],
    );
    let decomposed = graph(
        "main.veac",
        &[("main.veac", "e\u{301}", SourceAuthority::Project)],
        &[],
    );
    assert_ne!(composed.complete_revision(), decomposed.complete_revision());
}
