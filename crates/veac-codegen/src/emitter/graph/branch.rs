use std::collections::{BTreeMap, BTreeSet};

use super::{Graph, Node};

impl Graph {
    pub fn branch(&self, output: &str) -> Self {
        let mut required = BTreeSet::from([output.to_owned()]);
        let mut selected = vec![false; self.nodes.len()];
        let mut unused_outputs = Vec::new();
        let mut aliases = BTreeMap::new();
        for (index, node) in self.nodes.iter().enumerate().rev() {
            let used: Vec<_> = node
                .outputs
                .iter()
                .filter(|value| required.contains(*value))
                .cloned()
                .collect();
            if used.is_empty() {
                continue;
            }
            if node.is_transparent_fanout() && used.len() == 1 {
                aliases.insert(used[0].clone(), node.inputs[0].clone());
                required.insert(node.inputs[0].clone());
                continue;
            }
            selected[index] = true;
            unused_outputs.extend(
                node.outputs
                    .iter()
                    .filter(|value| !required.contains(*value))
                    .cloned(),
            );
            required.extend(node.inputs.iter().cloned());
        }
        let mut branch = Self {
            nodes: self
                .nodes
                .iter()
                .zip(selected)
                .filter(|(_, keep)| *keep)
                .map(|(node, _)| node.with_aliased_inputs(&aliases))
                .collect(),
            next_label: self.next_label,
        };
        for output in unused_outputs {
            branch.push(&[&output], "nullsink", Vec::new());
        }
        branch
    }
}

impl Node {
    fn is_transparent_fanout(&self) -> bool {
        self.inputs.len() == 1
            && self.outputs.len() > 1
            && ["split=", "asplit="].iter().any(|prefix| {
                self.expression
                    .strip_prefix(prefix)
                    .and_then(|value| value.parse().ok())
                    == Some(self.outputs.len())
            })
    }

    fn with_aliased_inputs(&self, aliases: &BTreeMap<String, String>) -> Self {
        let mut node = self.clone();
        for input in &mut node.inputs {
            while let Some(replacement) = aliases.get(input).cloned() {
                *input = replacement;
            }
        }
        node
    }
}
