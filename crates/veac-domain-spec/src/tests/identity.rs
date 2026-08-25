use std::collections::BTreeSet;

use super::*;

#[test]
fn v8_operation_identity_is_closed_unique_and_byte_stable() {
    let values = DomainOperationId::all().collect::<Vec<_>>();
    assert_eq!(values.len(), 582);
    assert_eq!(values.first(), Some(&DomainOperationId::Canvas));
    assert_eq!(values.last(), Some(&DomainOperationId::DeliverableHls));
    assert_eq!(DomainOperationId::Canvas.opcode(), 0x1001);
    assert_eq!(DomainOperationId::Project.opcode(), 0x201e);
    assert_eq!(DomainOperationId::GeneratorSolid.opcode(), 0x302c);
    assert_eq!(DomainOperationId::ItemWithVisual.opcode(), 0x403d);
    let effect_identities = [
        (DomainOperationId::VideoColorAdjustEffect, 0x5048),
        (DomainOperationId::VideoBlurEffect, 0x5049),
        (DomainOperationId::VideoSharpenEffect, 0x504a),
        (DomainOperationId::VideoVignetteEffect, 0x504b),
        (DomainOperationId::VideoGrainEffect, 0x504c),
        (DomainOperationId::VideoChromaKeyEffect, 0x504d),
        (DomainOperationId::VideoLumaKeyEffect, 0x504e),
        (DomainOperationId::VideoChromaSpillEffect, 0x504f),
        (DomainOperationId::VideoStabilizeEffect, 0x5050),
        (DomainOperationId::AudioNormalizeEffect, 0x5051),
        (DomainOperationId::ItemWithEffect, 0x5052),
        (DomainOperationId::PluginReferenceMonochromeV1, 0x5053),
        (DomainOperationId::VideoPluginScalarEffect, 0x5054),
        (DomainOperationId::VideoDirectionalBlurEffect, 0x5055),
    ];
    for (id, opcode) in effect_identities {
        assert_eq!(id.opcode(), opcode, "{}", id.name());
    }
    assert_eq!(DomainOperationId::RelationTransition.opcode(), 0x8018);
    assert_eq!(DomainOperationId::DeliverableHls.opcode(), 0xb028);

    let mut opcodes = BTreeSet::new();
    let mut names = BTreeSet::new();
    for id in values {
        assert!(opcodes.insert(id.opcode()));
        assert!(names.insert(id.name()));
        assert_eq!(DomainOperationId::from_opcode(id.opcode()), Some(id));
        assert_eq!(id.to_string(), format!("0x{:04x}", id.opcode()));
    }
    assert_eq!(DomainOperationId::from_opcode(0), None);
    assert_eq!(DomainOperationId::from_opcode(0xffff), None);
}

#[test]
fn v8_domain_type_identity_is_closed_unique_and_byte_stable() {
    let values = DomainType::all().collect::<Vec<_>>();
    assert_eq!(values.len(), 214);
    assert_eq!(values.first(), Some(&DomainType::Context));
    assert_eq!(values.last(), Some(&DomainType::HlsRendition));
    assert_eq!(DomainType::Context.opcode(), 0x0001);
    assert_eq!(DomainType::Canvas.opcode(), 0x1001);
    assert_eq!(DomainType::Project.opcode(), 0x2010);
    assert_eq!(DomainType::Transform2D.opcode(), 0x4009);
    assert_eq!(DomainType::HlsRendition.opcode(), 0xb00c);

    let mut opcodes = BTreeSet::new();
    let mut names = BTreeSet::new();
    for value in values {
        assert!(opcodes.insert(value.opcode()));
        assert!(names.insert(value.name()));
        assert_eq!(DomainType::from_opcode(value.opcode()), Some(value));
        assert_eq!(DomainType::parse(value.name()), Some(value));
        assert_eq!(value.to_string(), value.name());
        assert_eq!(format!("{value:?}"), value.name());
    }
    assert_eq!(DomainType::from_opcode(0), None);
    assert_eq!(DomainType::from_opcode(0xffff), None);
    assert_eq!(DomainType::parse("project"), None);
    assert_eq!(DomainType::parse("Unknown"), None);
}
