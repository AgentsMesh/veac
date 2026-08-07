use super::super::id;

#[test]
fn every_canonical_entity_id_wrapper_executes_in_the_test_build() {
    let path = ["project", "main", "owner"];
    let ids = [
        id::project(&path).to_string(),
        id::sequence(&path).to_string(),
        id::track(&path).to_string(),
        id::item(&path).to_string(),
        id::relation(&path).to_string(),
        id::material(&path).to_string(),
        id::multicam(&path).to_string(),
        id::angle(&path).to_string(),
        id::annotation(&path).to_string(),
        id::apply(&path).to_string(),
        id::apply_stage(&path).to_string(),
        id::delivery(&path).to_string(),
        id::deliverable(&path).to_string(),
        id::rendition(&path).to_string(),
        id::effect(&path).to_string(),
        id::keyframe(&path).to_string(),
        id::audio_processor(&path).to_string(),
        id::eq_band(&path).to_string(),
    ];
    let prefixes = [
        "prj_", "seq_", "trk_", "itm_", "rel_", "med_", "mcg_", "ang_", "ann_", "apl_", "aps_",
        "out_", "dlv_", "rnd_", "fx_", "kf_", "aud_", "eqb_",
    ];
    for (value, prefix) in ids.iter().zip(prefixes) {
        assert!(value.starts_with(prefix));
        assert_eq!(value.len(), prefix.len() + 64);
    }
}
