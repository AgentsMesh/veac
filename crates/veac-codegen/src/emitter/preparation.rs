use std::path::PathBuf;

use super::{
    BackendCommand, BackendFilterBinding, BackendFilterEscape, BackendInternalAccess,
    BackendPreparation, CodegenErrors, EmitContext,
};

impl EmitContext<'_> {
    pub(super) fn prepare_stabilization(
        &mut self,
        input: &str,
        range: Option<(&str, &str)>,
    ) -> Result<String, CodegenErrors> {
        let index = self.preparations.len();
        let path = PathBuf::from(format!("stabilize-{index:04}.trf"));
        let mut graph = self.graph.clone();
        let input = match range {
            Some((start, end)) => graph.filter(
                &[input],
                format!("trim=start={start}:end={end},setpts=PTS-STARTPTS"),
                "stabilizeanalysis",
            ),
            None => input.to_owned(),
        };
        let input = graph.filter(&[&input], "format=yuv444p16le", "stabilizeanalysis");
        let token = self.next_filter_token();
        let mut bindings = self.filter_bindings.clone();
        bindings.push(BackendFilterBinding::internal_file(
            token.clone(),
            path.clone(),
            BackendInternalAccess::Produce,
            BackendFilterEscape::Quoted,
        ));
        let output = graph.filter(
            &[&input],
            format!(
                "vidstabdetect=result='{token}':shakiness=10:accuracy=15:stepsize=4:mincontrast=0.01"
            ),
            "stabilizeanalysis",
        );
        let graph = graph.branch(&output);
        let (filter_graph, filter_contract) = self.render_preparation_graph(&graph, bindings)?;
        self.preparations.push(BackendPreparation {
            command: BackendCommand {
                preparations: Vec::new(),
                inputs: Vec::new(),
                filter_graph: Some(filter_graph),
                filter_contract: Some(filter_contract),
                maps: vec![format!("[{output}]")],
                output_args: vec!["-an".into(), "-f".into(), "null".into()],
                output_path: PathBuf::from(format!("stabilize-{index:04}.null")),
            },
            outputs: vec![path.clone()],
        });
        Ok(self.filter_internal_file(path, BackendInternalAccess::Consume))
    }

    pub(super) fn take_preparations(
        &mut self,
        inputs: &[super::BackendInput],
    ) -> Vec<BackendPreparation> {
        let mut values = std::mem::take(&mut self.preparations);
        for value in &mut values {
            value.command.inputs = inputs.to_vec();
        }
        values
    }
}
