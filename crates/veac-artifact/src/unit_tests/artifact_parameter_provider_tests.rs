use serde_json::json;

use crate::*;

#[test]
fn provider_slots_are_nominal_bounded_and_revalidated() {
    let slot = ProviderArtifactSlot::new("speech.zh-CN_1").unwrap();
    assert_eq!(slot.as_str(), "speech.zh-CN_1");
    assert_eq!(
        ProviderResultParameters::new("translation-main")
            .unwrap()
            .slot
            .as_str(),
        "translation-main"
    );

    for invalid in ["", "has space", "slash/name", "中文"] {
        assert_eq!(
            ProviderArtifactSlot::new(invalid).unwrap_err().kind,
            ArtifactErrorKind::InvalidContract
        );
    }
    assert_eq!(
        ProviderArtifactSlot::new("x".repeat(MAX_ARTIFACT_JSON_STRING_BYTES + 1))
            .unwrap_err()
            .kind,
        ArtifactErrorKind::ResourceLimit
    );

    let unchecked: ProviderArtifactSlot = serde_json::from_value(json!("bad slot")).unwrap();
    let parameters = ProviderResultParameters { slot: unchecked };
    assert_eq!(
        parameters.validate().unwrap_err().kind,
        ArtifactErrorKind::InvalidContract
    );
}

#[test]
fn provider_result_dispatch_is_closed_and_typed() {
    let supported = [
        ArtifactKind::Speech,
        ArtifactKind::Translation,
        ArtifactKind::MotionTrack,
        ArtifactKind::Matte,
        ArtifactKind::AudioStem,
        ArtifactKind::VideoMaster,
    ];
    for kind in supported {
        let value = ArtifactParameters::provider_result(
            kind,
            ProviderResultParameters::new(format!("{kind:?}")).unwrap(),
        )
        .unwrap();
        assert_eq!(value.kind(), kind);
        assert_eq!(value.provider_slot().unwrap().as_str(), format!("{kind:?}"));
        value.validate().unwrap();
    }

    let parameters = ProviderResultParameters::new("unsupported").unwrap();
    assert_eq!(
        ArtifactParameters::provider_result(ArtifactKind::ProxyVideo, parameters)
            .unwrap_err()
            .kind,
        ArtifactErrorKind::InvalidContract
    );
    assert!(
        ArtifactParameters::AudioFile(RenderOutputParameters::new(0, "audio.wav"))
            .provider_slot()
            .is_none()
    );
}

#[test]
fn provider_parameter_json_rejects_untyped_extensions() {
    let value = ArtifactParameters::Speech(ProviderResultParameters::new("voice").unwrap());
    let encoded = serde_json::to_value(&value).unwrap();
    assert_eq!(
        serde_json::from_value::<ArtifactParameters>(encoded).unwrap(),
        value
    );
    assert_eq!(
        serde_json::from_str::<ProviderArtifactSlot>(r#""voice""#)
            .unwrap()
            .as_str(),
        "voice"
    );

    assert!(serde_json::from_value::<ProviderResultParameters>(json!({})).is_err());
    assert!(serde_json::from_value::<ProviderResultParameters>(json!({
        "slot": "voice",
        "extension": true
    }))
    .is_err());
    assert!(serde_json::from_value::<ProducedArtifactParameters>(json!({
        "origin": "render",
        "parameters": {"index": 0, "path": "master.mov", "extension": true}
    }))
    .is_err());
    assert!(serde_json::from_value::<ArtifactParameters>(json!({
        "type": "speech",
        "parameters": {"slot": "voice", "extension": true}
    }))
    .is_err());
}
