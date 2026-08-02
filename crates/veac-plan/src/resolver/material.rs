use veac_ir::{Material, MaterialId, MaterialKind, MaterialSource};

use super::{streams::resolved_streams, PlanResolver};
use crate::{
    PlanInputId, ResolutionDiagnostic, ResolutionErrorKind, ResolvedInput, ResolvedInputKind,
    ResolvedMediaProbe,
};

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct InputUsage {
    pub video: bool,
    pub audio: bool,
    pub font: bool,
}

impl PlanResolver<'_> {
    pub(super) fn material_input(
        &mut self,
        material_id: &MaterialId,
        usage: InputUsage,
    ) -> Option<ResolvedInput> {
        let input_id = material_input_id(material_id).map_err(|error| {
            self.push_internal("PLAN_INPUT_ID", material_id.to_string(), error.to_string())
        });
        let Ok(input_id) = input_id else {
            return None;
        };
        if let Some(input) = self.inputs.get(&input_id).cloned() {
            self.check_usage(&input, usage);
            return Some(input);
        }
        let material = self
            .envelope
            .project
            .materials
            .iter()
            .find(|item| item.id == *material_id)
            .cloned()?;
        let input = self.build_input(input_id.clone(), &material, usage)?;
        self.inputs.insert(input_id, input.clone());
        Some(input)
    }

    fn build_input(
        &mut self,
        id: PlanInputId,
        material: &Material,
        usage: InputUsage,
    ) -> Option<ResolvedInput> {
        let path = format!("/project/materials/{}", material.id);
        let uri = match &material.source {
            MaterialSource::File { uri } => uri.clone(),
            MaterialSource::Remote { .. } => {
                self.push(
                    ResolutionDiagnostic::new(
                        ResolutionErrorKind::RemoteMaterialUnresolved,
                        "REMOTE_MATERIAL_UNRESOLVED",
                        Some(material.id.to_string()),
                        format!("{path}/source"),
                        "used remote material has no deterministic local materialization",
                    )
                    .repair("materialize the remote object and author a content identity"),
                );
                return None;
            }
        };
        let Some(identity) = material.identity.clone() else {
            let kind = if material.kind == MaterialKind::Font {
                ResolutionErrorKind::FontMaterialUnresolved
            } else {
                ResolutionErrorKind::MaterialIdentityMissing
            };
            self.push(ResolutionDiagnostic::new(
                kind,
                "MATERIAL_IDENTITY_MISSING",
                Some(material.id.to_string()),
                format!("{path}/identity"),
                "used material must have an authored content identity",
            ));
            return None;
        };
        if let Some(input) =
            super::resource::input(id.clone(), material, uri.clone(), identity.clone())
        {
            return Some(input);
        }
        let Some(probe) = &material.probe else {
            self.push(ResolutionDiagnostic::new(
                ResolutionErrorKind::MaterialProbeMissing,
                "MATERIAL_PROBE_MISSING",
                Some(material.id.to_string()),
                format!("{path}/probe"),
                "used audio, video, or image material requires a normalized probe snapshot",
            ));
            return None;
        };
        if probe.observed_identity != identity {
            self.push(ResolutionDiagnostic::new(
                ResolutionErrorKind::MaterialProbeIdentityMismatch,
                "MATERIAL_PROBE_IDENTITY_MISMATCH",
                Some(material.id.to_string()),
                format!("{path}/probe/observed_identity"),
                "probe facts were produced from different material bytes",
            ));
            return None;
        }
        let (video, audio) = resolved_streams(probe);
        if probe.selected_video_stream.is_some() != video.is_some()
            || probe.selected_audio_stream.is_some() != audio.is_some()
        {
            self.push(ResolutionDiagnostic::new(
                ResolutionErrorKind::StreamSelectionMismatch,
                "STREAM_SELECTION_MISMATCH",
                Some(material.id.to_string()),
                format!("{path}/probe"),
                "selected stream does not match the normalized stream inventory",
            ));
            return None;
        }
        let input = ResolvedInput {
            id,
            material_id: Some(material.id.clone()),
            kind: ResolvedInputKind::Media {
                material_kind: material.kind,
            },
            canonical_uri: uri,
            observed_identity: identity,
            probe: Some(ResolvedMediaProbe {
                schema_version: probe.schema_version,
                engine: probe.engine.clone(),
                selection_policy: probe.selection_policy.clone(),
                container_format: probe.container_format.clone(),
                container_duration: probe.container_duration,
            }),
            video,
            audio,
        };
        self.check_usage(&input, usage);
        Some(input)
    }

    fn check_usage(&mut self, input: &ResolvedInput, usage: InputUsage) {
        let material_id = input.material_id.as_ref().map(ToString::to_string);
        if usage.video && input.video.is_none() {
            self.push(ResolutionDiagnostic::new(
                ResolutionErrorKind::RequiredStreamMissing,
                "REQUIRED_VIDEO_STREAM_MISSING",
                material_id.clone(),
                format!(
                    "/project/materials/{}/probe",
                    material_id.as_deref().unwrap_or("unknown")
                ),
                "active visual clip requires a selected playable video stream",
            ));
        }
        if usage.audio && input.audio.is_none() {
            self.push(ResolutionDiagnostic::new(
                ResolutionErrorKind::RequiredStreamMissing,
                "REQUIRED_AUDIO_STREAM_MISSING",
                material_id,
                "/project/materials",
                "active audio clip requires a selected audio stream",
            ));
        }
        if usage.font && !matches!(&input.kind, ResolvedInputKind::Font { .. }) {
            self.push_internal(
                "FONT_INPUT_KIND",
                input.id.to_string(),
                "font input is not a font".into(),
            );
        }
    }
}

fn material_input_id(id: &MaterialId) -> Result<PlanInputId, crate::PlanIdError> {
    let suffix = id.as_str().strip_prefix("med_").unwrap_or(id.as_str());
    PlanInputId::new(format!("pin_{suffix}"))
}
