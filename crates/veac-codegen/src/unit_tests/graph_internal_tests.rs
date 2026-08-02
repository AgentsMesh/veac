use super::Graph;

#[test]
fn branch_elides_a_redundant_split_without_retaining_unrelated_consumers() {
    let mut graph = Graph::default();
    let source = graph.source("color=black", "source");
    let (selected, sibling) = graph.split(&source, "split");
    let output = graph.filter(&[&selected], "hflip", "selected");
    graph.filter(&[&sibling], "vflip", "unrelated");

    assert_eq!(
        graph.branch(&output).render_with_inputs(&[], &[]),
        "color=black[source0];[source0]hflip[selected3]"
    );
}

#[test]
fn branch_elides_nested_redundant_splits() {
    let mut graph = Graph::default();
    let source = graph.source("color=black", "source");
    let (selected, outer_sibling) = graph.split(&source, "outer");
    let (nested_selected, nested_sibling) = graph.split(&selected, "nested");
    let output = graph.filter(&[&nested_selected], "hflip", "selected");
    graph.filter(&[&nested_sibling], "vflip", "nested-unrelated");
    graph.filter(&[&outer_sibling], "negate", "outer-unrelated");

    assert_eq!(
        graph.branch(&output).render_with_inputs(&[], &[]),
        "color=black[source0];[source0]hflip[selected5]"
    );
}

#[test]
fn branch_sinks_unused_nontransparent_outputs() {
    let mut graph = Graph::default();
    let source = graph.source("color=black", "source");
    let planes = graph.filter_many(&[&source], "extractplanes=r+g", "plane", 2);
    let output = graph.filter(&[&planes[0]], "hflip", "selected");

    assert_eq!(
        graph.branch(&output).render_with_inputs(&[], &[]),
        "color=black[source0];[source0]extractplanes=r+g[plane1][plane2];\
         [plane1]hflip[selected3];[plane2]nullsink"
    );
}

#[test]
fn branch_does_not_sink_nested_split_siblings_when_all_are_selected() {
    let mut graph = Graph::default();
    let source = graph.source("color=black", "source");
    let (selected, outer_sibling) = graph.split(&source, "outer");
    let (first, second) = graph.split(&selected, "nested");
    let output = graph.filter(
        &[&first, &second, &outer_sibling],
        "hstack=inputs=3",
        "joined",
    );

    let rendered = graph.branch(&output).render_with_inputs(&[], &[]);
    assert_eq!(
        rendered,
        "color=black[source0];[source0]split=2[outer1][outer2];\
         [outer1]split=2[nested3][nested4];\
         [nested3][nested4][outer2]hstack=inputs=3[joined5]"
    );
    assert!(!rendered.contains("nullsink"), "{rendered}");
}
