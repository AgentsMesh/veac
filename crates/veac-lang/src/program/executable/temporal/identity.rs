use sha2::{Digest, Sha256};

use super::ExecutableTemporalLeaf;
use crate::program::expression::CompiledFunction;

const DOMAIN: &[u8] = b"veac.executable-v1.declared-inputs\0";

pub(in crate::program::executable) fn declared_inputs(
    main: &CompiledFunction,
    leaves: &[ExecutableTemporalLeaf],
    build_inputs_sha256: Option<&str>,
) -> String {
    let mut digest = Sha256::new();
    digest.update(DOMAIN);
    digest.update((leaves.len() as u64 + 1).to_be_bytes());
    digest.update([0x00]);
    digest.update(main.body().input_declarations_digest().as_bytes());
    if let Some(value) = build_inputs_sha256 {
        digest.update([0x02]);
        digest.update(value.as_bytes());
    }
    let mut leaves = leaves.iter().collect::<Vec<_>>();
    leaves.sort_by(|left, right| {
        left.sink()
            .cmp(right.sink())
            .then_with(|| left.binding_id().cmp(right.binding_id()))
    });
    for leaf in leaves {
        let identity = leaf.sink().identity_bytes();
        digest.update([0x01]);
        digest.update((identity.len() as u64).to_be_bytes());
        digest.update(identity);
        digest.update(
            leaf.expression()
                .core()
                .input_declarations_digest()
                .as_bytes(),
        );
    }
    format!("{:x}", digest.finalize())
}
