use std::collections::BTreeMap;

use veac_artifact::{BoundResource, BoundStream, ExecutionBindings, MediaRole, SourceClock};
use veac_plan::canonical::{Deliverable, RationalTime};
use veac_plan::{PlanInputId, ResolvedInputKind, ResolvedRenderPlan};

use super::{BackendInput, CodegenErrors};

mod contract;
mod errors;
mod timing;
pub(super) mod usage;

use errors::{invalid, missing};

#[derive(Debug, Clone)]
pub(super) struct RoutedStream {
    input_index: usize,
    physical_stream: veac_plan::canonical::StreamSelection,
    clock: SourceClock,
    source_duration: Option<RationalTime>,
    time_invariant: bool,
}

#[derive(Debug, Clone, Default)]
pub(super) struct InputRoutes {
    backend_inputs: Vec<BackendInput>,
    physical_resources: Vec<BoundResource>,
    resources: Vec<BoundResource>,
    video: BTreeMap<PlanInputId, RoutedStream>,
    audio: BTreeMap<PlanInputId, RoutedStream>,
}

pub(super) fn resolve(
    plan: &ResolvedRenderPlan,
    bindings: &ExecutionBindings,
    deliverable: &Deliverable,
) -> Result<InputRoutes, CodegenErrors> {
    let mut routes = InputRoutes::default();
    let usage = usage::required(plan, deliverable);
    for input in &plan.inputs {
        let video = usage.contains(&input.id, MediaRole::Video);
        let audio = usage.contains(&input.id, MediaRole::Audio);
        let resource = usage.resources().contains(&input.id);
        if !video && !audio && !resource {
            continue;
        }
        let Some(binding) = bindings.input(&input.id) else {
            return Err(missing(&input.id, None));
        };
        contract::source(input, binding)?;
        if is_media(input) {
            if video {
                let stream = binding
                    .video()
                    .ok_or_else(|| missing(&input.id, Some("video")))?;
                contract::stream(input, binding, MediaRole::Video, stream)?;
                routes.bind(input, MediaRole::Video, stream)?;
            }
            if audio {
                let stream = binding
                    .audio()
                    .ok_or_else(|| missing(&input.id, Some("audio")))?;
                contract::stream(input, binding, MediaRole::Audio, stream)?;
                routes.bind(input, MediaRole::Audio, stream)?;
            }
        } else if resource {
            let resource = binding
                .resource()
                .ok_or_else(|| missing(&input.id, Some("resource")))?;
            contract::resource(input, resource)?;
            routes.protect(resource)?;
        }
    }
    if bindings
        .inputs()
        .keys()
        .any(|id| !plan.inputs.iter().any(|input| input.id == *id))
    {
        return Err(invalid(
            "execution bindings contain an input outside the plan",
        ));
    }
    Ok(routes)
}

impl InputRoutes {
    pub fn backend_inputs(&self) -> &[BackendInput] {
        &self.backend_inputs
    }

    pub fn resources(&self) -> &[BoundResource] {
        &self.resources
    }

    pub fn stream(&self, id: &PlanInputId, role: MediaRole) -> Option<&RoutedStream> {
        match role {
            MediaRole::Video => self.video.get(id),
            MediaRole::Audio => self.audio.get(id),
        }
    }

    fn bind(
        &mut self,
        input: &veac_plan::ResolvedInput,
        role: MediaRole,
        stream: &BoundStream,
    ) -> Result<(), CodegenErrors> {
        let id = &input.id;
        let input_index = self.physical_index(stream.resource())?;
        let route = RoutedStream {
            input_index,
            physical_stream: stream.physical_stream(),
            clock: timing::clock(input, role, stream)?,
            source_duration: timing::duration(input, role),
            time_invariant: timing::time_invariant(input),
        };
        match role {
            MediaRole::Video => self.video.insert(id.clone(), route),
            MediaRole::Audio => self.audio.insert(id.clone(), route),
        };
        Ok(())
    }

    fn physical_index(&mut self, resource: &BoundResource) -> Result<usize, CodegenErrors> {
        if let Some((index, existing)) = self
            .physical_resources
            .iter()
            .enumerate()
            .find(|(_, value)| value.path() == resource.path())
        {
            if existing.identity() != resource.identity() {
                return Err(invalid(
                    "one resource path has conflicting expected identities",
                ));
            }
            return Ok(index);
        }
        self.protect(resource)?;
        let index = self.backend_inputs.len();
        self.backend_inputs.push(BackendInput {
            path: resource.path().to_owned(),
        });
        self.physical_resources.push(resource.clone());
        Ok(index)
    }

    fn protect(&mut self, resource: &BoundResource) -> Result<(), CodegenErrors> {
        if let Some(existing) = self
            .resources
            .iter()
            .find(|value| value.path() == resource.path())
        {
            if existing.identity() != resource.identity() {
                return Err(invalid(
                    "one resource path has conflicting expected identities",
                ));
            }
            return Ok(());
        }
        self.resources.push(resource.clone());
        Ok(())
    }
}

impl RoutedStream {
    pub fn input_index(&self) -> usize {
        self.input_index
    }

    pub fn global_stream(&self) -> u32 {
        self.physical_stream.global_index
    }

    pub fn clock(&self) -> SourceClock {
        self.clock
    }

    pub fn source_duration(&self) -> Option<RationalTime> {
        self.source_duration
    }

    pub fn time_invariant(&self) -> bool {
        self.time_invariant
    }
}

fn is_media(input: &veac_plan::ResolvedInput) -> bool {
    matches!(&input.kind, ResolvedInputKind::Media { .. })
}
