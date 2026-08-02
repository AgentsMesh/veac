use veac_plan::canonical::*;
use veac_plan::{
    resolve, resolve_one, ResolutionErrorKind, ResolvedClip, ResolvedMatte, ResolvedRenderPlan,
    ResolvedSidechain,
};

use super::{add_matte, add_sidechain, range};

pub fn add_relation_set(project: &mut ProjectEnvelope) {
    add_matte(
        project,
        "rel_matte_a",
        "seq_main",
        "itm_matte_a",
        "itm_video",
        matte_parameters(TrackMatteMode::Alpha, false),
    );
    add_matte(
        project,
        "rel_matte_b",
        "seq_main",
        "itm_matte_b",
        "itm_second",
        matte_parameters(TrackMatteMode::Luma, true),
    );
    add_sidechain(
        project,
        "rel_side_a",
        "seq_main",
        RelationEndpoint::bus(BusId::new("bus_dialogue").unwrap()),
        "itm_video",
        sidechain_parameters(-18.0, 4.0),
    );
    add_sidechain(
        project,
        "rel_side_b",
        "seq_main",
        RelationEndpoint::track(TrackId::new("trk_key").unwrap()),
        "itm_second",
        sidechain_parameters(-12.0, 2.0),
    );
}

pub fn matte_parameters(mode: TrackMatteMode, invert: bool) -> MatteRelationParameters {
    MatteRelationParameters { mode, invert }
}

pub fn sidechain_parameters(threshold_db: f64, ratio: f64) -> SidechainRelationParameters {
    SidechainRelationParameters {
        threshold_db,
        ratio,
        attack_ms: 10.0,
        release_ms: 250.0,
        active_range: Some(range(0, 300)),
    }
}

pub fn resolved(project: &ProjectEnvelope) -> ResolvedRenderPlan {
    resolve_one(project, &project.project.render_configs[0].id).unwrap()
}

pub fn clip<'a>(plan: &'a ResolvedRenderPlan, id: &str) -> &'a ResolvedClip {
    plan.sequences
        .iter()
        .flat_map(|sequence| &sequence.tracks)
        .flat_map(|track| &track.clips)
        .find(|clip| clip.id.as_str() == id)
        .unwrap()
}

pub fn relation_projection(
    plan: &ResolvedRenderPlan,
) -> Vec<(String, Option<ResolvedMatte>, Option<ResolvedSidechain>)> {
    let mut values: Vec<_> = plan
        .sequences
        .iter()
        .flat_map(|sequence| &sequence.tracks)
        .flat_map(|track| &track.clips)
        .map(|clip| {
            let matte = clip
                .visual
                .as_ref()
                .and_then(|value| value.track_matte.clone());
            let sidechain = clip
                .audio
                .as_ref()
                .and_then(|value| value.sidechain.clone());
            (clip.id.to_string(), matte, sidechain)
        })
        .collect();
    values.sort_by(|left, right| left.0.cmp(&right.0));
    values
}

pub fn assert_canonical_error(project: &ProjectEnvelope, code: &str) {
    let errors = resolve(project, None).unwrap_err();
    let diagnostic = errors
        .diagnostics()
        .iter()
        .find(|diagnostic| diagnostic.code == code)
        .unwrap();
    assert_eq!(diagnostic.kind, ResolutionErrorKind::CanonicalValidation);
}
