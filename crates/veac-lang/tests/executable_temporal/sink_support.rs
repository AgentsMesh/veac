pub const SOURCE: &str = r#"
fn main(context: Context) -> Project {
    let visual = item(
        identifier("visual-clip"), item_enabled(), during(0s, 3s),
        source_generated(generator_transparent()), source_timing_native()
    ).with_visual(visual_style(
        visual_layout(
            placement_anchor(anchor_center(), vector(0.0, 0.0)), frame_none(),
            transform_2d(
                transform_motion(
                    point_constant(point(0px, 0px)),
                    vector_constant(vector(1.0, 1.0)), angle_constant(0deg)
                ),
                transform_geometry(
                    vector(0.0, 0.0), flip_none(), vector(0.5, 0.5),
                    crop_animated(rect_constant(rect(0.0, 0.0, 1.0, 1.0)))
                )
            )
        ),
        visual_surface(
            percent_constant(100%), compositing(0, blend_normal()), card_none()
        ),
        [], color_pipeline_none()
    ));
    let audio = item(
        identifier("audio-clip"), item_enabled(), during(0s, 3s),
        source_generated(generator_silence()), source_timing_native()
    ).with_audio(audio_style(
        scalar_constant(1.0), scalar_constant(0.0),
        audio_playback(false, false, pitch_preserve()), [], audio_crossfade_none()
    ));
    let state = track_state(
        track_playback_enabled(), track_audio_audible(),
        track_isolation_normal(), track_editing_unlocked()
    );
    let visuals = visual_layer(
        identifier("visual"), 0, placement_free(), state, track_routing_default()
    ).with_item(visual);
    let audio_track = audio_layer(
        identifier("audio"), 1, placement_free(), state, track_routing_default()
    ).with_item(audio);
    let timeline = sequence(
        identifier("main"), "属性绑定矩阵",
        sequence_settings(canvas(320px, 180px), frame_rate(30, 1), 48000)
    ).with_layer(visuals).with_layer(audio_track);
    project(identifier("sink-matrix"), project_settings(600))
        .with_sequence(timeline).entry(timeline)
}
"#;

pub fn items() -> (veac_ir::ItemId, veac_ir::ItemId) {
    let built = veac_lang::program::build_source(SOURCE).unwrap();
    let sequence = &built.envelope().project.sequences[0];
    (
        sequence.tracks[0].clips[0].id.clone(),
        sequence.tracks[1].clips[0].id.clone(),
    )
}
