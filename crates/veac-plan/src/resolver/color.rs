use veac_ir::{ColorPipeline, ColorStage, MaterialKind, VisualProperties};

use super::{material::InputUsage, PlanResolver};
use crate::{
    EffectiveVisualProperties, ResolvedColorPipeline, ResolvedColorStage, ResolvedLut,
    ResolvedLutKind,
};

impl PlanResolver<'_> {
    pub(super) fn resolve_visual(
        &mut self,
        authored: Option<&VisualProperties>,
    ) -> EffectiveVisualProperties {
        let mut visual = super::defaults::visual(authored);
        visual.color_pipeline = authored
            .and_then(|value| value.color_pipeline.as_ref())
            .and_then(|pipeline| self.resolve_color_pipeline(pipeline));
        visual
    }

    pub(super) fn resolve_color_pipeline(
        &mut self,
        value: &ColorPipeline,
    ) -> Option<ResolvedColorPipeline> {
        let mut stages = Vec::with_capacity(value.stages.len());
        for stage in &value.stages {
            let resolved = match stage {
                ColorStage::Basic { adjustment } => ResolvedColorStage::Basic {
                    adjustment: *adjustment,
                },
                ColorStage::Matrix { adjustment } => ResolvedColorStage::Matrix {
                    adjustment: *adjustment,
                },
                ColorStage::Hsl { adjustment } => ResolvedColorStage::Hsl {
                    adjustment: *adjustment,
                },
                ColorStage::Curves { curves } => ResolvedColorStage::Curves {
                    curves: curves.clone(),
                },
                ColorStage::Wheels { wheels } => ResolvedColorStage::Wheels { wheels: *wheels },
                ColorStage::Lut { application } => {
                    let input =
                        self.material_input(&application.material_id, InputUsage::default())?;
                    let kind = match input.kind {
                        crate::ResolvedInputKind::Resource {
                            material_kind: MaterialKind::Lut1d,
                        } => ResolvedLutKind::OneDimensional,
                        crate::ResolvedInputKind::Resource {
                            material_kind: MaterialKind::Lut3d,
                        } => ResolvedLutKind::ThreeDimensional,
                        _ => {
                            self.push_internal(
                                "LUT_INPUT_KIND",
                                application.material_id.to_string(),
                                "validated LUT resolved to a non-LUT input".to_owned(),
                            );
                            return None;
                        }
                    };
                    ResolvedColorStage::Lut {
                        application: ResolvedLut {
                            input_id: input.id,
                            kind,
                            interpolation: application.interpolation,
                        },
                    }
                }
            };
            stages.push(resolved);
        }
        Some(ResolvedColorPipeline {
            input: value.input,
            working: value.working,
            output: value.output,
            stages,
        })
    }
}
