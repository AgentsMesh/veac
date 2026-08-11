use std::collections::{BTreeMap, BTreeSet};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use veac_runtime::observation as runtime;

use crate::{
    DecodeObservation, FrameObservation, LayerOrderObservation, ObservationPlanV1, ObservationSet,
    PixelFormat,
};

#[derive(Debug, Clone)]
pub struct BoundObservationSource {
    pub source_id: String,
    pub source: runtime::ObservationSource,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ObservationFailure {
    pub observation_id: String,
    pub message: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ObservationRun {
    pub observations: ObservationSet,
    pub failures: Vec<ObservationFailure>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObservationRunError {
    DuplicateBinding(String),
    MissingBinding(String),
    UnexpectedBinding(String),
}

pub trait ObservationBackend: Send + Sync {
    fn frame(&self, request: &runtime::FrameRequest) -> Result<runtime::DecodedFrame, String>;
    fn decode(
        &self,
        request: &runtime::DecodeRequest,
    ) -> Result<runtime::DecodeObservation, String>;
}

impl ObservationBackend for runtime::MediaObserver {
    fn frame(&self, request: &runtime::FrameRequest) -> Result<runtime::DecodedFrame, String> {
        self.frame(request).map_err(|error| error.to_string())
    }

    fn decode(
        &self,
        request: &runtime::DecodeRequest,
    ) -> Result<runtime::DecodeObservation, String> {
        self.decode(request).map_err(|error| error.to_string())
    }
}

pub fn execute_observations(
    plan: &ObservationPlanV1,
    bindings: &[BoundObservationSource],
    backend: &dyn ObservationBackend,
    layer_orders: BTreeMap<String, LayerOrderObservation>,
) -> Result<ObservationRun, ObservationRunError> {
    let required = required_sources(plan);
    let sources = index_bindings(bindings, &required)?;
    let mut run = ObservationRun {
        observations: ObservationSet {
            layer_orders,
            ..ObservationSet::default()
        },
        failures: Vec::new(),
    };
    for frame in &plan.frames {
        let request = runtime::FrameRequest {
            source: sources[&frame.source_id].clone(),
            time: frame.at,
            pixel_format: runtime::FramePixelFormat::Rgba8,
        };
        match backend.frame(&request) {
            Ok(value) => insert_frame(&mut run, frame, value),
            Err(message) => run.failures.push(ObservationFailure {
                observation_id: frame.sample_ids.join(","),
                message,
            }),
        }
    }
    for decode in &plan.decodes {
        let request = runtime::DecodeRequest {
            source: sources[&decode.source_id].clone(),
        };
        match backend.decode(&request) {
            Ok(value) => insert_decode(&mut run, &decode.source_id, value),
            Err(message) => {
                run.observations.decodes.insert(
                    decode.source_id.clone(),
                    DecodeObservation {
                        complete: false,
                        decoded_frames: 0,
                        last_pts: None,
                        errors: vec![message.clone()],
                    },
                );
                run.failures.push(ObservationFailure {
                    observation_id: decode.source_id.clone(),
                    message,
                });
            }
        }
    }
    Ok(run)
}

fn insert_frame(
    run: &mut ObservationRun,
    plan: &crate::PlannedFrame,
    value: runtime::DecodedFrame,
) {
    let frame = FrameObservation {
        width: value.width,
        height: value.height,
        format: PixelFormat::Rgba8,
        actual_pts: value.actual_pts,
        data: value.bytes,
    };
    for id in &plan.sample_ids {
        run.observations.frames.insert(id.clone(), frame.clone());
    }
}

fn insert_decode(run: &mut ObservationRun, id: &str, value: runtime::DecodeObservation) {
    run.observations.decodes.insert(
        id.to_owned(),
        DecodeObservation {
            complete: value.complete,
            decoded_frames: value.decoded_frames,
            last_pts: value.last_pts,
            errors: value.errors,
        },
    );
}

fn required_sources(plan: &ObservationPlanV1) -> BTreeSet<&str> {
    plan.frames
        .iter()
        .map(|value| value.source_id.as_str())
        .chain(plan.decodes.iter().map(|value| value.source_id.as_str()))
        .collect()
}

fn index_bindings<'a>(
    bindings: &'a [BoundObservationSource],
    required: &BTreeSet<&str>,
) -> Result<BTreeMap<String, &'a runtime::ObservationSource>, ObservationRunError> {
    let mut sources = BTreeMap::new();
    for binding in bindings {
        if !required.contains(binding.source_id.as_str()) {
            return Err(ObservationRunError::UnexpectedBinding(
                binding.source_id.clone(),
            ));
        }
        if sources
            .insert(binding.source_id.clone(), &binding.source)
            .is_some()
        {
            return Err(ObservationRunError::DuplicateBinding(
                binding.source_id.clone(),
            ));
        }
    }
    if let Some(missing) = required.iter().find(|id| !sources.contains_key(**id)) {
        return Err(ObservationRunError::MissingBinding((*missing).to_owned()));
    }
    Ok(sources)
}

impl std::fmt::Display for ObservationRunError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateBinding(id) => write!(formatter, "duplicate observation binding: {id}"),
            Self::MissingBinding(id) => write!(formatter, "missing observation binding: {id}"),
            Self::UnexpectedBinding(id) => {
                write!(formatter, "unexpected observation binding: {id}")
            }
        }
    }
}

impl std::error::Error for ObservationRunError {}
