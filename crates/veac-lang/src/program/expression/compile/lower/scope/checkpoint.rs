use super::{Checkpoint, Lowerer};

impl Lowerer<'_> {
    pub(in crate::program::expression::compile::lower) fn checkpoint(&self) -> Checkpoint {
        Checkpoint {
            next_local: self.next_local,
            next_mutable: self.next_mutable,
            capture_lengths: self
                .closures
                .iter()
                .map(|value| value.captures.len())
                .collect(),
        }
    }

    pub(in crate::program::expression::compile::lower) fn rollback(
        &mut self,
        checkpoint: &Checkpoint,
    ) {
        self.next_local = checkpoint.next_local;
        self.next_mutable = checkpoint.next_mutable;
        for (closure, length) in self.closures.iter_mut().zip(&checkpoint.capture_lengths) {
            closure.captures.truncate(*length);
            closure.keys.truncate(*length);
        }
    }
}
