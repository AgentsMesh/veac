use super::super::directory::Directory;
use crate::RuntimeError;

pub(in crate::executor) trait Observer {
    fn before_backup(&self, _target: &str) {}

    fn before_install(&self, _source: &str, _target: &str) {}

    fn sync_committed(&self, stage: &Directory) -> Result<(), RuntimeError> {
        stage.sync()
    }
}

pub(super) struct LiveObserver;

impl Observer for LiveObserver {}
