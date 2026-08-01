use super::*;

impl<'a> EmitContext<'a> {
    pub(super) fn new_visual(
        plan: &'a ResolvedRenderPlan,
        bindings: &'a ExecutionBindings,
        deliverable: &'a Deliverable,
        alpha: AlphaMode,
    ) -> Result<Self, CodegenErrors> {
        let Some(canvas) = Canvas::from_output(&plan.output) else {
            return Err(CodegenErrors::one(error::diagnostic(
                CodegenErrorKind::InvalidPlan,
                "PLAN_RASTER_REQUIRED",
                Some(deliverable.id.to_string()),
                "visual artifact requires delivery raster settings",
            )));
        };
        Self::create(plan, bindings, deliverable, alpha, canvas)
    }

    pub(super) fn new_audio(
        plan: &'a ResolvedRenderPlan,
        bindings: &'a ExecutionBindings,
        deliverable: &'a Deliverable,
    ) -> Result<Self, CodegenErrors> {
        let Some(sequence) = plan
            .sequences
            .iter()
            .find(|value| value.id == plan.entry_sequence_id)
        else {
            return Err(CodegenErrors::one(error::missing_entry(plan)));
        };
        Self::create(
            plan,
            bindings,
            deliverable,
            AlphaMode::Opaque,
            Canvas::from_sequence(sequence),
        )
    }

    fn create(
        plan: &'a ResolvedRenderPlan,
        bindings: &'a ExecutionBindings,
        deliverable: &'a Deliverable,
        alpha: AlphaMode,
        canvas: Canvas,
    ) -> Result<Self, CodegenErrors> {
        preflight::validate(plan)?;
        let input_routes = input::resolve(plan, bindings, deliverable)?;
        output::validate_binding(deliverable, bindings)?;
        Ok(Self {
            plan,
            bindings,
            deliverable,
            alpha,
            input_routes,
            canvas,
            graph: Graph::default(),
            filter_bindings: Vec::new(),
            preparations: Vec::new(),
        })
    }

    pub(super) fn build_video(
        mut self,
        settings: &VideoDeliverable,
    ) -> Result<BackendCommand, CodegenErrors> {
        let Some(sequence) = self
            .plan
            .sequences
            .iter()
            .find(|value| value.id == self.plan.entry_sequence_id)
        else {
            return Err(CodegenErrors::one(error::missing_entry(self.plan)));
        };
        let video = sequence::build_entry(&mut self, sequence)?;
        let video = sequence::conform_output(&mut self, video, sequence);
        let video = match settings.video.color_space {
            Some(space) => color_space::tag(&mut self, video, space),
            None => video,
        };
        let audio = match &settings.audio {
            Some(output) => Some(audio::build_audio(&mut self, sequence, output)?),
            None => None,
        };
        let inputs = self.input_routes.backend_inputs().to_vec();
        let output_path = output::bound_path(self.deliverable, self.bindings)?;
        let mut maps = vec![format!("[{video}]")];
        if let Some(audio) = audio {
            maps.push(format!("[{audio}]"));
        }
        let mut output_args = output::arguments(self.canvas, settings);
        output_args.extend(["-t".to_owned(), time::seconds(sequence.duration)]);
        let (filter_graph, filter_contract) = self.filter_graph()?;
        let preparations = self.take_preparations(&inputs);
        Ok(BackendCommand {
            preparations,
            inputs,
            filter_graph,
            filter_contract,
            maps,
            output_args,
            output_path,
        })
    }

    pub(super) fn captions_visible(&self) -> bool {
        self.plan
            .output
            .raster
            .as_ref()
            .is_some_and(|raster| raster.captions == veac_plan::canonical::CaptionOutput::BurnIn)
    }
}
