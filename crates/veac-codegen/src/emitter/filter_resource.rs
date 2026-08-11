use std::path::PathBuf;

use veac_artifact::MediaRole;

use super::error::{diagnostic, CodegenErrorKind};
use super::{
    graph::Graph, BackendFilterBinding, BackendFilterContract, BackendFilterEscape,
    BackendInternalAccess, CodegenErrors, EmitContext, MAX_FILTER_GRAPH_BYTES,
};

impl EmitContext<'_> {
    pub(super) fn filter_file(&mut self, path: PathBuf, escape: BackendFilterEscape) -> String {
        let token = self.next_filter_token();
        self.filter_bindings
            .push(BackendFilterBinding::file(token.clone(), path, escape));
        token
    }

    pub(super) fn filter_directory(
        &mut self,
        directory: PathBuf,
        files: Vec<PathBuf>,
        escape: BackendFilterEscape,
    ) -> String {
        let token = self.next_filter_token();
        self.filter_bindings.push(BackendFilterBinding::directory(
            token.clone(),
            directory,
            files,
            escape,
        ));
        token
    }

    pub(super) fn filter_internal_file(
        &mut self,
        path: PathBuf,
        access: BackendInternalAccess,
    ) -> String {
        let token = self.next_filter_token();
        self.filter_bindings
            .push(BackendFilterBinding::internal_file(
                token.clone(),
                path,
                access,
                BackendFilterEscape::Quoted,
            ));
        token
    }

    pub(super) fn filter_graph(
        &self,
    ) -> Result<(Option<String>, Option<BackendFilterContract>), CodegenErrors> {
        self.reverse_ledger
            .validate(self.deliverable, self.graph.reverse_filter_count())?;
        if self.graph.is_empty() {
            return Ok((None, None));
        }
        let (rendered, contract) = self.render_graph(&self.graph, self.filter_bindings.clone())?;
        Ok((Some(rendered), Some(contract)))
    }

    pub(super) fn render_graph(
        &self,
        graph: &Graph,
        bindings: Vec<BackendFilterBinding>,
    ) -> Result<(String, BackendFilterContract), CodegenErrors> {
        self.render_template(self.graph_template(graph), bindings)
    }

    pub(super) fn render_preparation_graph(
        &self,
        graph: &Graph,
        mut bindings: Vec<BackendFilterBinding>,
    ) -> Result<(String, BackendFilterContract), CodegenErrors> {
        let template = self.graph_template(graph);
        bindings.retain(|binding| template.contains(binding.token()));
        self.render_template(template, bindings)
    }

    fn graph_template(&self, graph: &Graph) -> String {
        let video = self.external_inputs(MediaRole::Video);
        let audio = self.external_inputs(MediaRole::Audio);
        graph.render_with_inputs(&video, &audio)
    }

    fn render_template(
        &self,
        template: String,
        bindings: Vec<BackendFilterBinding>,
    ) -> Result<(String, BackendFilterContract), CodegenErrors> {
        let contract = BackendFilterContract::new(template, bindings)
            .map_err(|message| self.invalid_filter(message))?;
        let rendered = contract
            .render_original()
            .map_err(|message| self.invalid_filter(message))?;
        if rendered.len() > MAX_FILTER_GRAPH_BYTES {
            return Err(self.invalid_filter(format!(
                "FFmpeg filter graph exceeds {MAX_FILTER_GRAPH_BYTES} bytes"
            )));
        }
        Ok((rendered, contract))
    }

    pub(super) fn next_filter_token(&self) -> String {
        format!("__VEAC_FILTER_RESOURCE_{:04}__", self.filter_bindings.len())
    }

    fn external_inputs(&self, role: MediaRole) -> Vec<String> {
        let mut labels: Vec<_> = self
            .plan
            .inputs
            .iter()
            .filter_map(|input| self.input_routes.stream(&input.id, role))
            .map(|route| format!("{}:{}", route.input_index(), route.global_stream()))
            .collect();
        labels.sort();
        labels.dedup();
        labels
    }

    fn invalid_filter(&self, message: impl Into<String>) -> CodegenErrors {
        CodegenErrors::one(diagnostic(
            CodegenErrorKind::InvalidPlan,
            "BACKEND_FILTER_GRAPH_INVALID",
            Some(self.plan.output.id.to_string()),
            message,
        ))
    }
}
