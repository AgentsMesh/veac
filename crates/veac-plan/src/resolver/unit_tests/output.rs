use super::{mechanism_helpers::text_clip, support::*};
use crate::{canonical::*, resolve};

#[test]
fn output_encoder_settings_are_preserved_in_the_plan() {
    let mut project = project();
    let output = &mut project.project.render_configs[0];
    let id = DeliverableId::new("dlv_main").unwrap();
    let delivery = output.video_deliverable_mut(&id).unwrap();
    delivery.video = VideoOutput {
        codec: VideoCodec::H264,
        pixel_format: PixelFormat::Yuv420p,
        alpha: AlphaMode::Opaque,
        color_space: Some(ColorSpace {
            primaries: ColorPrimaries::Bt709,
            transfer: ColorTransfer::Bt709,
            matrix: ColorMatrix::Bt709,
            range: ColorRange::Limited,
        }),
        rate_control: VideoRateControl::Bitrate {
            target_bps: 5_000_000,
            max_bps: Some(7_000_000),
            buffer_size_bits: Some(10_000_000),
        },
        gop_size: Some(48),
        b_frames: Some(4),
        profile: Some(VideoProfile::H264High),
        level: Some("5.1".to_owned()),
    };
    delivery.optimize_for_streaming = true;
    let expected = delivery.video.clone();
    let plan = resolve(&project, None).unwrap().remove(0);
    let (_, delivery) = plan.output.video_deliverable(&id).unwrap();
    assert_eq!(delivery.video, expected);
    assert!(delivery.optimize_for_streaming);
    assert_eq!(plan.output.raster.unwrap().captions, CaptionOutput::BurnIn);
}

#[test]
fn discard_caption_policy_removes_visual_caption_dependencies() {
    let mut project = project();
    project.project.render_configs[0]
        .raster
        .as_mut()
        .unwrap()
        .captions = CaptionOutput::Discard;
    project.project.materials.push(font_material("med_font"));
    project.project.sequences[0].tracks.push(track(
        "trk_caption",
        TrackKind::Caption,
        20,
        vec![text_clip(true)],
    ));
    let plan = resolve(&project, None).unwrap().remove(0);
    assert!(plan.inputs.iter().all(|input| input
        .material_id
        .as_ref()
        .is_none_or(|id| id.as_str() != "med_font")));
    let track = plan.sequences[0]
        .tracks
        .iter()
        .find(|track| track.id.as_str() == "trk_caption")
        .unwrap();
    assert!(!track.state.include_in_render);
    assert!(!track.state.visual_enabled);
    assert!(track.clips.is_empty());
}

#[test]
fn resolved_output_accessors_find_and_mutate_video_deliverables() {
    let mut output = resolve(&project(), None).unwrap().remove(0).output;
    let id = output.deliverables[0].id.clone();
    assert_eq!(output.deliverable(&id).unwrap().id, id);
    assert!(output
        .deliverable(&DeliverableId::new("dlv_missing").unwrap())
        .is_none());
    output.video_deliverable_mut(&id).unwrap().container = OutputFormat::Mov;
    assert_eq!(
        output.video_deliverable(&id).unwrap().1.container,
        OutputFormat::Mov
    );

    output.deliverables[0].kind = DeliverableKind::CaptionSidecar(CaptionSidecarOutput {
        format: CaptionSidecarFormat::WebVtt,
        track_ids: vec![],
    });
    assert!(output.video_deliverable(&id).is_none());
    assert!(output.video_deliverable_mut(&id).is_none());
}
