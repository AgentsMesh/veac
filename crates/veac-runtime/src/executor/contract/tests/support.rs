use std::path::Path;

use veac_codegen::emitter::{
    BackendAction, BackendOutput, BackendPhase, BackendProduct, BackendTask,
};
use veac_ir::{HashAlgorithm, MediaIdentity};

use crate::executor::model::RuntimeBundle as BackendBundle;

pub(super) fn bundle(output: &Path) -> BackendBundle {
    BackendBundle {
        plan_identity: veac_artifact::ContentDigest::sha256(b"contract-test-plan"),
        substitution_proof: veac_artifact::ContentDigest::sha256(b"test-substitution"),
        protected_resources: vec![],
        requirements: vec![],
        tasks: vec![BackendTask {
            deliverable_id: veac_ir::DeliverableId::new("dlv_contract_test").unwrap(),
            phase: BackendPhase::Single,
            product: BackendProduct::CaptionSidecar,
            output: BackendOutput::File(output.to_path_buf()),
            action: BackendAction::WriteFile {
                path: output.to_path_buf(),
                content: b"caption".to_vec(),
            },
        }],
    }
}

pub(super) fn identity(path: &Path) -> MediaIdentity {
    crate::asset::sha256_identity(path).unwrap()
}

pub(super) fn fake_identity(value: char) -> MediaIdentity {
    MediaIdentity {
        algorithm: HashAlgorithm::Sha256,
        digest: value.to_string().repeat(64),
    }
}
