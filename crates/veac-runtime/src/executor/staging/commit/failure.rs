use crate::RuntimeError;

pub(in crate::executor) struct CommitFailure {
    error: RuntimeError,
    preserve_staging: bool,
}

impl CommitFailure {
    pub(in crate::executor) fn direct(error: RuntimeError) -> Self {
        Self {
            error,
            preserve_staging: false,
        }
    }

    pub(in crate::executor) fn rollback(error: RuntimeError, rollback: RuntimeError) -> Self {
        let kind = error.kind;
        Self {
            error: RuntimeError {
                kind,
                message: format!("{error}; rollback failed: {rollback}"),
            },
            preserve_staging: true,
        }
    }

    pub(in crate::executor) fn preserve_staging(&self) -> bool {
        self.preserve_staging
    }

    pub(in crate::executor) fn into_error(self) -> RuntimeError {
        self.error
    }
}

impl From<RuntimeError> for CommitFailure {
    fn from(error: RuntimeError) -> Self {
        Self::direct(error)
    }
}
