use sha2::{Digest, Sha256};

use super::super::instruction;
use crate::program::expression::{CoreInstructionKind as Kind, LocalSlotId, ValueId};

fn encoded(kind: Kind) -> [u8; 32] {
    let mut digest = Sha256::new();
    instruction::encode(&mut digest, &kind);
    digest.finalize().into()
}

fn distinct(left: Kind, right: Kind) {
    assert_ne!(encoded(left), encoded(right));
}

#[test]
fn local_instruction_digest_covers_slot_and_value_operands() {
    let first_slot = LocalSlotId::new(0);
    let next_slot = LocalSlotId::new(1);
    let first_value = ValueId::new(0);
    let next_value = ValueId::new(1);
    distinct(
        Kind::LocalInit {
            slot: first_slot,
            value: first_value,
        },
        Kind::LocalInit {
            slot: next_slot,
            value: first_value,
        },
    );
    distinct(
        Kind::LocalInit {
            slot: first_slot,
            value: first_value,
        },
        Kind::LocalInit {
            slot: first_slot,
            value: next_value,
        },
    );
    distinct(
        Kind::LocalSet {
            slot: first_slot,
            value: first_value,
        },
        Kind::LocalSet {
            slot: next_slot,
            value: next_value,
        },
    );
    distinct(
        Kind::LocalGet { slot: first_slot },
        Kind::LocalGet { slot: next_slot },
    );
}
