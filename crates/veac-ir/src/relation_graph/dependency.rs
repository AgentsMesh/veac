use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Default)]
pub struct MatteDependencyGraph<N> {
    edges: BTreeMap<N, BTreeSet<N>>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MatteDependencyAnalysis {
    pub cyclic: bool,
    pub max_depth: usize,
}

impl<N: Ord + Clone> MatteDependencyGraph<N> {
    pub fn new() -> Self {
        Self {
            edges: BTreeMap::new(),
        }
    }

    pub fn add(&mut self, consumer: N, producer: N) {
        self.edges
            .entry(consumer.clone())
            .or_default()
            .insert(producer.clone());
        self.edges.entry(producer).or_default();
    }

    pub fn analyze(&self) -> MatteDependencyAnalysis {
        let mut indegree: BTreeMap<_, usize> =
            self.edges.keys().cloned().map(|node| (node, 0)).collect();
        for producers in self.edges.values() {
            for producer in producers {
                *indegree.entry(producer.clone()).or_default() += 1;
            }
        }
        let mut ready: BTreeSet<_> = indegree
            .iter()
            .filter_map(|(node, degree)| (*degree == 0).then_some(node.clone()))
            .collect();
        let mut depths: BTreeMap<N, usize> = BTreeMap::new();
        let mut processed = 0;
        let mut max_depth = 0;
        while let Some(node) = ready.pop_first() {
            processed += 1;
            let depth = *depths.get(&node).unwrap_or(&0);
            max_depth = max_depth.max(depth);
            for producer in self.edges.get(&node).into_iter().flatten() {
                depths
                    .entry(producer.clone())
                    .and_modify(|value| *value = (*value).max(depth + 1))
                    .or_insert(depth + 1);
                let degree = indegree
                    .get_mut(producer)
                    .expect("registered dependency node");
                *degree -= 1;
                if *degree == 0 {
                    ready.insert(producer.clone());
                }
            }
        }
        MatteDependencyAnalysis {
            cyclic: processed != indegree.len(),
            max_depth,
        }
    }
}
