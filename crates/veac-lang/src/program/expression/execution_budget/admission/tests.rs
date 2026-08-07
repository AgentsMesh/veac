use super::{add_container, add_delta, LOGICAL_COLLECTION_HANDLE_BYTES};
use crate::program::expression::execution_budget::ResourceDelta;

#[test]
fn recursive_admission_arithmetic_never_wraps() {
    let mut slot_product = ResourceDelta::default();
    assert!(add_container(&mut slot_product, usize::MAX, 2).is_none());

    let mut handle_product = ResourceDelta::default();
    assert!(add_container(
        &mut handle_product,
        usize::MAX / LOGICAL_COLLECTION_HANDLE_BYTES + 1,
        1,
    )
    .is_none());

    let mut bytes = ResourceDelta {
        collection_bytes: usize::MAX,
        ..ResourceDelta::default()
    };
    assert!(add_container(&mut bytes, 0, 1).is_none());

    let mut elements = ResourceDelta {
        collection_elements: usize::MAX,
        ..ResourceDelta::default()
    };
    assert!(add_container(&mut elements, 1, 1).is_none());

    let mut payload = ResourceDelta {
        value_bytes: usize::MAX,
        ..ResourceDelta::default()
    };
    assert!(add_delta(
        &mut payload,
        ResourceDelta {
            value_bytes: 1,
            ..ResourceDelta::default()
        }
    )
    .is_none());

    for mut target in [
        ResourceDelta {
            collection_elements: usize::MAX,
            ..ResourceDelta::default()
        },
        ResourceDelta {
            collection_bytes: usize::MAX,
            ..ResourceDelta::default()
        },
    ] {
        assert!(add_delta(
            &mut target,
            ResourceDelta {
                collection_elements: 1,
                collection_bytes: 1,
                ..ResourceDelta::default()
            }
        )
        .is_none());
    }
}
