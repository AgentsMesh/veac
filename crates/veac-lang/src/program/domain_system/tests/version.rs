use super::*;

#[test]
fn version_and_digest_value_apis_are_closed() {
    assert_eq!(DomainOpsetVersion::from_raw(1), DomainOpsetVersion::V1);
    assert_eq!(DomainOpsetVersion::V1.raw(), 1);
    assert_eq!(DomainOpsetVersion::V1.to_string(), "1");
    assert_eq!(DomainOpsetVersion::from_raw(2), DomainOpsetVersion::V2);
    assert_eq!(DomainOpsetVersion::V2.raw(), 2);
    assert_eq!(DomainOpsetVersion::from_raw(3), DomainOpsetVersion::V3);
    assert_eq!(DomainOpsetVersion::V3.raw(), 3);
    assert_eq!(DomainOpsetVersion::from_raw(4), DomainOpsetVersion::V4);
    assert_eq!(DomainOpsetVersion::V4.raw(), 4);
    assert_eq!(DomainOpsetVersion::from_raw(5), DomainOpsetVersion::V5);
    assert_eq!(DomainOpsetVersion::V5.raw(), 5);
    assert_eq!(DomainOpsetVersion::from_raw(6), DomainOpsetVersion::V6);
    assert_eq!(DomainOpsetVersion::V6.raw(), 6);
    assert_eq!(DomainOpsetVersion::from_raw(7), DomainOpsetVersion::V7);
    assert_eq!(DomainOpsetVersion::V7.raw(), 7);
    assert_eq!(DomainOpsetVersion::CURRENT, DomainOpsetVersion::V7);
    let digest = DomainRegistryDigest::from_bytes([0xab; 32]);
    assert_eq!(digest.as_bytes(), &[0xab; 32]);
    assert_eq!(digest.to_string(), "ab".repeat(32));
}
