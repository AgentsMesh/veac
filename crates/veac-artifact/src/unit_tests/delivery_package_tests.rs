use crate::*;

#[test]
fn package_inventory_identity_is_stable_and_sensitive_to_every_member_field() {
    let value = inventory(vec![
        file("master.m3u8", b"master"),
        file("segment.ts", b"one"),
    ]);
    value.validate().unwrap();
    assert_eq!(value, inventory(value.members.clone()));

    let mut path = value.members.clone();
    path[1].path = "segment-2.ts".into();
    assert_ne!(value.tree, inventory(path).tree);
    let content = inventory(vec![
        file("master.m3u8", b"master"),
        file("segment.ts", b"two"),
    ]);
    assert_ne!(value.tree, content.tree);
}

#[test]
fn package_inventory_accepts_directories_and_sums_file_bytes() {
    let value = inventory(vec![
        file("master.m3u8", b"master"),
        directory("segments"),
        file("segments/one.ts", b"one"),
    ]);

    assert_eq!(value.size_bytes(), 9);
    value.validate().unwrap();
}

#[test]
fn package_inventory_rejects_unsafe_unsorted_duplicate_and_malformed_members() {
    for members in [
        vec![file("../master.m3u8", b"bad")],
        vec![file("z.ts", b"z"), file("master.m3u8", b"master")],
        vec![file("master.m3u8", b"one"), file("master.m3u8", b"two")],
        vec![DeliveryPackageMember {
            path: "master.m3u8".into(),
            node_type: DeliveryPackageNodeType::RegularFile,
            size_bytes: 0,
            content: Some(ContentDigest::sha256(b"")),
        }],
        vec![DeliveryPackageMember {
            path: "master.m3u8".into(),
            node_type: DeliveryPackageNodeType::Directory,
            size_bytes: 1,
            content: None,
        }],
    ] {
        assert!(DeliveryPackageInventory::new("master.m3u8", members).is_err());
    }
}

#[test]
fn package_inventory_rejects_missing_entrypoint_and_tampered_identity() {
    let error = DeliveryPackageInventory::new("missing.m3u8", vec![file("master.m3u8", b"master")])
        .unwrap_err();
    assert_eq!(error.kind, ArtifactErrorKind::InvalidContract);

    let mut value = inventory(vec![file("master.m3u8", b"master")]);
    value.tree = ContentDigest::sha256(b"tampered");
    let error = value.validate().unwrap_err();
    assert_eq!(error.kind, ArtifactErrorKind::InvalidContract);
}

#[test]
fn package_inventory_enforces_count_and_byte_budgets() {
    let empty = DeliveryPackageInventory::new("master.m3u8", Vec::new()).unwrap_err();
    assert_eq!(empty.kind, ArtifactErrorKind::ResourceLimit);

    let over_budget = DeliveryPackageInventory::new(
        "master.m3u8",
        vec![sized_file("master.m3u8", MAX_RENDER_TASK_OUTPUT_BYTES + 1)],
    )
    .unwrap_err();
    assert_eq!(over_budget.kind, ArtifactErrorKind::ResourceLimit);

    let overflow = DeliveryPackageInventory::new(
        "master.m3u8",
        vec![
            sized_file("master.m3u8", u64::MAX),
            sized_file("segment.ts", 1),
        ],
    )
    .unwrap_err();
    assert_eq!(overflow.kind, ArtifactErrorKind::ResourceLimit);
}

fn inventory(mut members: Vec<DeliveryPackageMember>) -> DeliveryPackageInventory {
    members.sort_by(|left, right| left.path.cmp(&right.path));
    DeliveryPackageInventory::new("master.m3u8", members).unwrap()
}

fn file(path: &str, bytes: &[u8]) -> DeliveryPackageMember {
    DeliveryPackageMember {
        path: path.into(),
        node_type: DeliveryPackageNodeType::RegularFile,
        size_bytes: bytes.len() as u64,
        content: Some(ContentDigest::sha256(bytes)),
    }
}

fn sized_file(path: &str, size_bytes: u64) -> DeliveryPackageMember {
    DeliveryPackageMember {
        path: path.into(),
        node_type: DeliveryPackageNodeType::RegularFile,
        size_bytes,
        content: Some(ContentDigest::sha256(b"payload")),
    }
}

fn directory(path: &str) -> DeliveryPackageMember {
    DeliveryPackageMember {
        path: path.into(),
        node_type: DeliveryPackageNodeType::Directory,
        size_bytes: 0,
        content: None,
    }
}
