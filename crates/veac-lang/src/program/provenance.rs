use std::collections::BTreeMap;

use crate::authoring::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Origin {
    pub path: String,
    pub span: Span,
    pub instance_id: Option<String>,
    pub definition: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProvenanceMap {
    entries: BTreeMap<String, Origin>,
}

impl ProvenanceMap {
    pub fn get(&self, expanded_id: &str) -> Option<&Origin> {
        self.entries.get(expanded_id)
    }

    pub fn get_local(&self, instance: &str, local: &str) -> Option<&Origin> {
        self.get(&super::expand::hygiene::encode(instance, local))
    }

    pub fn get_local_path(&self, instance: &str, locals: &[&str]) -> Option<&Origin> {
        self.get(&super::expand::hygiene::encode_path(instance, locals))
    }

    pub(crate) fn try_insert(&mut self, id: String, origin: Origin) -> bool {
        if let std::collections::btree_map::Entry::Vacant(entry) = self.entries.entry(id) {
            entry.insert(origin);
            return true;
        }
        false
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &Origin)> {
        self.entries
            .iter()
            .map(|(id, origin)| (id.as_str(), origin))
    }
}
