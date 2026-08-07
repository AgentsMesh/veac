use crate::*;

pub(super) fn descriptor(
    kind: ArtifactKind,
    mut dependencies: Vec<ArtifactDependency>,
    tag: u8,
) -> ArtifactDescriptor {
    dependencies.sort_by(|left, right| {
        (&left.role, &left.identity.value).cmp(&(&right.role, &right.identity.value))
    });
    let parameters = match kind {
        ArtifactKind::ProxyVideo => proxy_video(tag),
        ArtifactKind::ProxyAudio => proxy_audio(tag),
        ArtifactKind::RenderSegment => render_segment(tag),
        _ => unreachable!(),
    };
    ArtifactDescriptor::new(crate::test_support::producer(), dependencies, parameters)
}

pub(super) fn dependency(role: ArtifactDependencyRole, bytes: &[u8]) -> ArtifactDependency {
    ArtifactDependency::new(role, ContentDigest::sha256(bytes))
}

pub(super) fn cache_directory(root: &std::path::Path, key: &ContentDigest) -> std::path::PathBuf {
    root.join("sha256")
        .join(&key.value[..2])
        .join(&key.value[2..])
}

fn proxy_video(tag: u8) -> ArtifactParameters {
    let ArtifactParameters::ProxyVideo(mut value) = crate::test_support::descriptor().parameters
    else {
        unreachable!()
    };
    value.crf = 20 + tag;
    ArtifactParameters::ProxyVideo(value)
}

fn proxy_audio(tag: u8) -> ArtifactParameters {
    ArtifactParameters::ProxyAudio(ProxyAudioSpec {
        source_stream: veac_ir::StreamSelection {
            global_index: 0,
            type_index: 0,
        },
        source_clock: SourceClockSpec::Identity {
            duration: veac_ir::RationalTime::new(10, 10).unwrap(),
        },
        sample_rate: 44_100 + u32::from(tag),
        channels: 2,
    })
}

fn render_segment(tag: u8) -> ArtifactParameters {
    ArtifactParameters::RenderSegment(RenderSegmentParameters {
        sequence_id: veac_ir::SequenceId::new("seq_main").unwrap(),
        range: veac_ir::TimeRange::new(
            veac_ir::RationalTime::zero(10).unwrap(),
            veac_ir::RationalTime::new(i64::from(tag), 10).unwrap(),
        )
        .unwrap(),
        deliverable_id: veac_ir::DeliverableId::new("dlv_main").unwrap(),
        fidelity: RenderSegmentFidelity::ExactDeliveryMaster,
    })
}
