use std::collections::{BTreeMap, VecDeque};
use std::fmt::Write as _;

mod branch;

#[derive(Debug, Clone, Default)]
pub(crate) struct Graph {
    nodes: Vec<Node>,
    next_label: usize,
}

#[derive(Debug, Clone)]
struct Node {
    inputs: Vec<String>,
    expression: String,
    outputs: Vec<String>,
}

impl Graph {
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub(crate) fn reverse_filter_count(&self) -> u64 {
        self.nodes
            .iter()
            .flat_map(|node| node.expression.split(','))
            .filter(|filter| {
                let name = filter.trim().split(['@', '=']).next().unwrap_or_default();
                matches!(name, "reverse" | "areverse")
            })
            .count() as u64
    }

    pub fn filter(&mut self, inputs: &[&str], expression: impl AsRef<str>, prefix: &str) -> String {
        let output = self.label(prefix);
        self.push(inputs, expression, vec![output.clone()]);
        output
    }

    pub fn filter_many(
        &mut self,
        inputs: &[&str],
        expression: impl AsRef<str>,
        prefix: &str,
        outputs: usize,
    ) -> Vec<String> {
        let labels: Vec<_> = (0..outputs).map(|_| self.label(prefix)).collect();
        self.push(inputs, expression, labels.clone());
        labels
    }

    pub fn split(&mut self, input: &str, prefix: &str) -> (String, String) {
        let mut labels = self.filter_many(&[input], "split=2", prefix, 2);
        (labels.remove(0), labels.remove(0))
    }

    pub fn source(&mut self, expression: impl AsRef<str>, prefix: &str) -> String {
        self.filter(&[], expression, prefix)
    }

    pub fn render_with_inputs(&self, video: &[String], audio: &[String]) -> String {
        let mut external = BTreeMap::new();
        external.extend(
            video
                .iter()
                .cloned()
                .map(|label| (label, ("split", "inputsplitv"))),
        );
        external.extend(
            audio
                .iter()
                .cloned()
                .map(|label| (label, ("asplit", "inputsplita"))),
        );
        let mut next_label = self.next_label;
        let mut replacements = BTreeMap::new();
        let mut rendered = Vec::new();
        for (input, (filter, prefix)) in external {
            let uses = self.input_uses(&input);
            if uses < 2 {
                continue;
            }
            let outputs: Vec<_> = (0..uses)
                .map(|_| {
                    let label = format!("{prefix}{next_label}");
                    next_label += 1;
                    label
                })
                .collect();
            rendered.push(render_node(
                std::slice::from_ref(&input),
                &format!("{filter}={uses}"),
                &outputs,
            ));
            replacements.insert(input, VecDeque::from(outputs));
        }
        rendered.extend(self.nodes.iter().map(|node| {
            let inputs = node
                .inputs
                .iter()
                .map(|input| {
                    replacements
                        .get_mut(input)
                        .and_then(VecDeque::pop_front)
                        .unwrap_or_else(|| input.clone())
                })
                .collect::<Vec<_>>();
            render_node(&inputs, &node.expression, &node.outputs)
        }));
        rendered.join(";")
    }

    fn push(&mut self, inputs: &[&str], expression: impl AsRef<str>, outputs: Vec<String>) {
        self.nodes.push(Node {
            inputs: inputs.iter().map(|value| (*value).to_owned()).collect(),
            expression: expression.as_ref().to_owned(),
            outputs,
        });
    }

    fn input_uses(&self, input: &str) -> usize {
        self.nodes
            .iter()
            .flat_map(|node| &node.inputs)
            .filter(|value| value.as_str() == input)
            .count()
    }

    fn label(&mut self, prefix: &str) -> String {
        let label = format!("{prefix}{}", self.next_label);
        self.next_label += 1;
        label
    }
}

fn render_node(inputs: &[String], expression: &str, outputs: &[String]) -> String {
    let mut rendered = render_labels(inputs.iter().map(String::as_str));
    rendered.push_str(expression);
    rendered.push_str(&render_labels(outputs.iter().map(String::as_str)));
    rendered
}

fn render_labels<'a>(values: impl IntoIterator<Item = &'a str>) -> String {
    values.into_iter().fold(String::new(), |mut output, value| {
        write!(output, "[{value}]").expect("writing to a String cannot fail");
        output
    })
}

#[cfg(test)]
#[path = "../unit_tests/graph_internal_tests.rs"]
mod tests;
