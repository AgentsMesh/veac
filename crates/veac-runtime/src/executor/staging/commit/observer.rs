pub(in crate::executor) trait Observer {
    fn before_backup(&self, _target: &str) {}

    fn before_install(&self, _source: &str, _target: &str) {}
}

pub(super) struct LiveObserver;

impl Observer for LiveObserver {}
