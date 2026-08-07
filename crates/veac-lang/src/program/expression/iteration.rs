use std::ops::Range;
use std::sync::Arc;

use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LoopLogicalKey(Arc<str>);

impl LoopLogicalKey {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpressionLoopFrame {
    logical_key: LoopLogicalKey,
    definition: Arc<str>,
    function: Arc<str>,
    source: Arc<str>,
    loop_span: Range<usize>,
    binding_span: Range<usize>,
    index: usize,
}

impl ExpressionLoopFrame {
    pub(crate) fn new(
        definition: Arc<str>,
        function: Arc<str>,
        source: Arc<str>,
        loop_span: Range<usize>,
        binding_span: Range<usize>,
        index: usize,
    ) -> Self {
        let logical_key = logical_key(&definition, &loop_span, &binding_span);
        Self {
            logical_key,
            definition,
            function,
            source,
            loop_span,
            binding_span,
            index,
        }
    }

    pub fn logical_key(&self) -> &LoopLogicalKey {
        &self.logical_key
    }

    pub fn definition(&self) -> &str {
        &self.definition
    }

    pub fn function(&self) -> &str {
        &self.function
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn loop_span(&self) -> Range<usize> {
        self.loop_span.clone()
    }

    pub fn binding_span(&self) -> Range<usize> {
        self.binding_span.clone()
    }

    pub const fn index(&self) -> usize {
        self.index
    }
}

fn logical_key(
    definition: &str,
    loop_span: &Range<usize>,
    binding_span: &Range<usize>,
) -> LoopLogicalKey {
    let mut digest = Sha256::new();
    digest.update(b"veac.loop-logical-key.v1");
    field(&mut digest, definition.as_bytes());
    for value in [
        loop_span.start,
        loop_span.end,
        binding_span.start,
        binding_span.end,
    ] {
        digest.update((value as u64).to_be_bytes());
    }
    LoopLogicalKey(format!("loop_{:x}", digest.finalize()).into())
}

fn field(digest: &mut Sha256, value: &[u8]) {
    digest.update((value.len() as u64).to_be_bytes());
    digest.update(value);
}

#[cfg(test)]
#[path = "iteration/tests.rs"]
mod tests;
