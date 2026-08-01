mod validation;

use veac_codegen::emitter::{emit_all, BackendAction};
use veac_plan::canonical::*;
use veac_plan::{ResolvedClipSource, ResolvedMulticamSource, ResolvedRenderPlan};

use super::support::{bindings, fixture, identity, probe, resolved, time};

#[test]
fn multicam_switches_map_two_resources_and_concatenate_video_and_audio() {
    let plan = multicam_plan();
    assert_eq!(plan.inputs.len(), 2);
    assert_eq!(multicam_source(&plan).angles.len(), 2);
    assert!(plan.inputs.iter().all(|input| input
        .material_id
        .as_ref()
        .is_none_or(|id| id.as_str() != "med_unused")));
    let bindings = bindings(&plan);
    let bundle = emit_all(&plan, &bindings).unwrap();
    assert_eq!(bundle.protected_resources().len(), 2);
    for input in &plan.inputs {
        let path = bindings
            .input(&input.id)
            .unwrap()
            .resource()
            .unwrap()
            .path();
        let resource = bundle
            .protected_resources()
            .iter()
            .find(|resource| resource.path == path)
            .unwrap();
        assert_eq!(resource.expected_identity, input.observed_identity);
    }
    let command = command(&bundle.tasks()[0]);
    assert_eq!(command.inputs.len(), 2);
    for (actual, input) in command.inputs.iter().zip(&plan.inputs) {
        assert_eq!(
            actual.path,
            bindings
                .input(&input.id)
                .unwrap()
                .resource()
                .unwrap()
                .path()
        );
    }
    assert_eq!(command.maps.len(), 2);
    assert!(pair(&command.output_args, "-c:a", "aac"));
    assert!(pair(&command.output_args, "-ar", "48000"));

    let graph = command.filter_graph.as_deref().unwrap();
    let source = multicam_source(&plan);
    for (switch, expected_start) in source.switches.iter().zip(["0", "0.6"]) {
        let angle = source
            .angles
            .iter()
            .find(|angle| angle.id == switch.angle_id)
            .unwrap();
        let input = plan
            .inputs
            .iter()
            .position(|input| input.id == angle.input_id)
            .unwrap();
        assert!(graph.contains(&format!(
            "[{input}:{}]trim=start={expected_start}:duration=0.5,setpts=PTS-STARTPTS",
            angle.video_stream.global_index
        )));
        assert!(graph.contains(&format!(
            "[{input}:{}]atrim=start={expected_start}:duration=0.5,asetpts=PTS-STARTPTS,aresample=48000",
            angle.audio_stream.unwrap().global_index
        )));
    }
    assert_eq!(graph.matches("fps=30/1").count(), 2);
    assert!(graph.contains("concat=n=2:v=1:a=0"));
    assert!(graph.contains("concat=n=2:v=0:a=1"));
}

pub(crate) fn multicam_plan() -> ResolvedRenderPlan {
    let mut project = fixture();
    let mut wide = project.project.materials[0].clone();
    wide.id = MaterialId::new("med_wide").unwrap();
    wide.source = MaterialSource::File {
        uri: "media/wide.mp4".to_owned(),
    };
    let observed = identity('b');
    wide.identity = Some(observed.clone());
    wide.probe = Some(probe(observed));
    project.project.materials.push(wide);
    let mut unused = project.project.materials[0].clone();
    unused.id = MaterialId::new("med_unused").unwrap();
    unused.source = MaterialSource::File {
        uri: "media/unused-missing.mp4".to_owned(),
    };
    unused.identity = None;
    unused.probe = None;
    project.project.materials.push(unused);
    project.project.multicam_groups.push(group());
    project.project.render_configs[0]
        .video_deliverable_mut(&DeliverableId::new("dlv_main").unwrap())
        .unwrap()
        .audio = Some(AudioOutput {
        codec: AudioCodec::Aac,
        sample_rate: 48_000,
        channels: 2,
    });
    let clip = &mut project.project.sequences[0].tracks[0].clips[0];
    clip.source = ClipSource::Multicam {
        group_id: MulticamGroupId::new("mcg_cameras").unwrap(),
        switches: switches(),
    };
    clip.source_mapping = None;
    clip.audio = Some(AudioProperties {
        gain: Animatable::constant(1.0),
        pan: Animatable::constant(0.0),
        muted: false,
        normalize: false,
        pitch_policy: PitchPolicy::Preserve,
        processors: vec![],
        crossfade: None,
    });
    resolved(&project)
}
fn group() -> MulticamGroup {
    MulticamGroup {
        id: MulticamGroupId::new("mcg_cameras").unwrap(),
        sync: MulticamSync {
            basis: MulticamSyncBasis::Manual,
            reference_angle_id: MulticamAngleId::new("ang_close").unwrap(),
        },
        angles: vec![
            angle("ang_close", "med_video", 0),
            angle("ang_unused", "med_unused", 0),
            angle("ang_wide", "med_wide", 60),
        ],
    }
}
fn angle(id: &str, material: &str, offset: i64) -> MulticamAngle {
    MulticamAngle {
        id: MulticamAngleId::new(id).unwrap(),
        material_id: MaterialId::new(material).unwrap(),
        source_offset: time(offset),
    }
}
fn switches() -> Vec<MulticamSwitch> {
    vec![switch("ang_close", 0, 300), switch("ang_wide", 300, 300)]
}
fn switch(id: &str, start: i64, duration: i64) -> MulticamSwitch {
    MulticamSwitch {
        angle_id: MulticamAngleId::new(id).unwrap(),
        range: TimeRange::new(time(start), time(duration)).unwrap(),
    }
}

fn multicam_source(plan: &ResolvedRenderPlan) -> &ResolvedMulticamSource {
    let ResolvedClipSource::Multicam { source } = &plan.sequences[0].tracks[0].clips[0].source
    else {
        unreachable!()
    };
    source
}

fn command(task: &veac_codegen::emitter::BackendTask) -> &veac_codegen::emitter::BackendCommand {
    let BackendAction::Ffmpeg(command) = &task.action else {
        unreachable!()
    };
    command
}

fn pair(arguments: &[String], name: &str, value: &str) -> bool {
    arguments.windows(2).any(|pair| pair == [name, value])
}
