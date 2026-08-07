use veac_artifact::*;

use crate::environment::Environment;

pub(in crate::unit_tests) fn proxy_descriptor(
    prepared: &crate::planning::PreparedPlan,
    environment: &dyn Environment,
) -> ArtifactDescriptor {
    let input = &prepared.plan.inputs[0];
    let stream = input.video.as_ref().unwrap().selection;
    let raster = prepared.plan.output.raster.as_ref().unwrap();
    MediaArtifactRequest {
        source_identity: ContentDigest {
            algorithm: DigestAlgorithm::Sha256,
            value: input.observed_identity.digest.clone(),
        },
        producer: producer(environment),
        spec: MediaArtifactSpec::ProxyVideo(ProxyVideoSpec {
            source_stream: stream,
            source_clock: SourceClockSpec::Identity {
                duration: normalized_time(
                    input
                        .video
                        .as_ref()
                        .and_then(|value| value.duration)
                        .or_else(|| {
                            input
                                .probe
                                .as_ref()
                                .and_then(|value| value.container_duration)
                        })
                        .unwrap(),
                    prepared.plan.header.source.timebase,
                ),
            },
            width: raster.width,
            height: raster.height,
            frame_rate: raster.frame_rate,
            crf: 28,
        }),
    }
    .descriptor()
    .unwrap()
}

fn normalized_time(value: veac_ir::RationalTime, timescale: u32) -> veac_ir::RationalTime {
    let numerator = i128::from(value.value) * i128::from(timescale);
    assert_eq!(numerator % i128::from(value.timescale), 0);
    veac_ir::RationalTime::new(
        i64::try_from(numerator / i128::from(value.timescale)).unwrap(),
        timescale,
    )
    .unwrap()
}

pub(in crate::unit_tests) fn segment_contract(
    prepared: &crate::planning::PreparedPlan,
    environment: &dyn Environment,
) -> FullRenderSegmentContract {
    FullRenderSegmentContract::new(
        &prepared.plan,
        prepared.bindings.input_substitution_proof(),
        producer(environment),
    )
    .unwrap()
}

fn producer(environment: &dyn Environment) -> ProducerFingerprint {
    let value = environment.ffmpeg_fingerprint().unwrap();
    veac_runtime::workflow::media_artifact_producer(&value).unwrap()
}
