use super::super::*;

pub(super) fn multicam_request() -> ProviderRequest {
    let angle = |angle_id: &str, material_id: &str| MulticamSyncInput {
        angle_id: veac_ir::MulticamAngleId::new(angle_id).unwrap(),
        material_id: veac_ir::MaterialId::new(material_id).unwrap(),
        media: input(MediaType::Audio),
        timecode_start: None,
    };
    ProviderRequest::MulticamSync(MulticamSyncRequest {
        basis: veac_ir::MulticamSyncBasis::Audio,
        reference_angle_id: veac_ir::MulticamAngleId::new("ang_a").unwrap(),
        angles: vec![
            angle("ang_a", "med_source_video"),
            angle("ang_b", "med_source_video_b"),
        ],
        maximum_offset: time(100),
    })
}
