use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::sync::Arc;

use super::{CompiledFunction, FunctionId};

#[derive(Debug, Clone, Default)]
pub(crate) struct FunctionRegistry {
    functions: BTreeMap<FunctionId, Arc<CompiledFunction>>,
}

impl FunctionRegistry {
    pub(crate) fn iter_with_ids(
        &self,
    ) -> impl Iterator<Item = (FunctionId, &Arc<CompiledFunction>)> {
        self.functions.iter().map(|(id, function)| (*id, function))
    }

    pub(crate) fn get(&self, id: FunctionId) -> Option<&Arc<CompiledFunction>> {
        self.functions.get(&id)
    }

    pub(crate) fn insert(&mut self, function: Arc<CompiledFunction>) {
        self.functions.entry(function.id()).or_insert(function);
    }

    pub(crate) fn merge(&mut self, other: &Self) {
        for function in other.functions.values() {
            self.insert(Arc::clone(function));
        }
    }

    pub(crate) fn reachable<'a>(&self, roots: impl IntoIterator<Item = &'a FunctionId>) -> Self {
        let mut retained = Self::default();
        let mut pending = roots.into_iter().copied().collect::<Vec<_>>();
        let mut visited = BTreeSet::new();
        while let Some(id) = pending.pop() {
            if !visited.insert(id) {
                continue;
            }
            let Some(function) = self.get(id) else {
                continue;
            };
            pending.extend(function.body().called_functions());
            pending.extend(
                function
                    .parameters()
                    .iter()
                    .filter_map(|parameter| parameter.default().map(|value| value.thunk())),
            );
            retained.insert(Arc::clone(function));
        }
        retained
    }
}
