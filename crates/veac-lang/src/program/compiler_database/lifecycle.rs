use std::sync::Arc;

use super::CompilerDatabase;

#[derive(Clone, Debug)]
pub(crate) struct QueryEpoch(Arc<()>, Arc<()>);

#[derive(Debug)]
pub(super) struct Lifecycle {
    epoch: Arc<()>,
    release: Arc<()>,
}

impl Lifecycle {
    pub(super) fn new() -> Self {
        Self {
            epoch: Arc::new(()),
            release: Arc::new(()),
        }
    }

    pub(super) fn epoch(&self) -> QueryEpoch {
        QueryEpoch(Arc::clone(&self.epoch), Arc::clone(&self.release))
    }

    pub(super) fn admits(&self, epoch: &QueryEpoch) -> bool {
        Arc::ptr_eq(&self.epoch, &epoch.0)
    }

    pub(super) fn same_release(&self, epoch: &QueryEpoch) -> bool {
        Arc::ptr_eq(&self.release, &epoch.1)
    }

    pub(super) fn advance(&mut self) {
        self.epoch = Arc::new(());
    }

    fn release(&mut self) {
        self.advance();
        self.release = Arc::new(());
    }
}

impl CompilerDatabase {
    pub(crate) fn query_epoch(&self) -> QueryEpoch {
        self.lifecycle
            .read()
            .expect("compiler database lifecycle lock poisoned")
            .epoch()
    }

    pub fn clear(&self) {
        let mut lifecycle = self
            .lifecycle
            .write()
            .expect("compiler database lifecycle lock poisoned");
        lifecycle.release();
        self.syntax
            .lock()
            .expect("syntax cache lock poisoned")
            .clear();
        self.interface
            .lock()
            .expect("interface cache lock poisoned")
            .clear();
        self.hir.lock().expect("HIR cache lock poisoned").clear();
        self.core.lock().expect("core cache lock poisoned").clear();
        self.dependencies
            .lock()
            .expect("dependency graph lock poisoned")
            .clear();
    }
}
