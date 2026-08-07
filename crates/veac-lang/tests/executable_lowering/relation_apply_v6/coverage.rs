use veac_ir::{RelationEndpoint, RelationKind, TrackMatteMode};

use super::super::support;
use super::SOURCE;

#[test]
fn sidechain_bus_and_inverted_luma_matte_lower() {
    let source = SOURCE
        .replace(
            "let video = video_layer(",
            "let bus = audio_bus(identifier(\"duck\"));\n    let video = video_layer(",
        )
        .replace(
            "let music_track = audio_layer(identifier(\"music\"), 3, placement_free(), state,\n        track_routing_default())",
            "let music_track = audio_layer(identifier(\"music\"), 3, placement_free(), state,\n        track_routing_bus(bus))",
        )
        .replace(
            "relation_matte_apply(\n        identifier(\"matte\"), matte, grade, matte_alpha(), false)",
            "relation_matte_apply(\n        identifier(\"matte\"), matte, grade, matte_luma(), true)",
        )
        .replace(
            "relation_sidechain_track(\n        identifier(\"sidechain\"), music_track, voice,",
            "relation_sidechain_bus(\n        identifier(\"sidechain\"), bus, voice,",
        );
    let envelope = support::envelope(&source);
    assert!(envelope.project.relations.iter().any(|relation| matches!(
        &relation.kind,
        RelationKind::Matte { parameters, .. }
            if parameters.mode == TrackMatteMode::Luma && parameters.invert
    )));
    assert!(envelope.project.relations.iter().any(|relation| matches!(
        &relation.kind,
        RelationKind::Sidechain {
            key: RelationEndpoint::Bus { .. },
            ..
        }
    )));
}
