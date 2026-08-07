use sha2::{Digest, Sha256};

use super::super::instruction;
use crate::program::expression::{BlockId, CoreMatchArm, CoreTerminator, ValueId};
use crate::program::VariantIndex;

#[test]
fn terminator_digest_covers_every_control_transfer_shape() {
    let values = [
        CoreTerminator::Return {
            value: ValueId::new(1),
            span: 0..1,
        },
        CoreTerminator::Jump {
            target: BlockId::new(2),
            arguments: vec![ValueId::new(1)],
            span: 0..1,
        },
        CoreTerminator::Branch {
            condition: ValueId::new(1),
            then_target: BlockId::new(2),
            else_target: BlockId::new(3),
            span: 0..1,
        },
        CoreTerminator::Match {
            scrutinee: ValueId::new(1),
            arms: vec![CoreMatchArm {
                variant: VariantIndex::new(0),
                target: BlockId::new(2),
            }],
            span: 0..1,
        },
    ];
    let outputs = values
        .iter()
        .map(|value| {
            let mut digest = Sha256::new();
            instruction::terminator(&mut digest, value);
            <[u8; 32]>::from(digest.finalize())
        })
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(outputs.len(), values.len());
}
