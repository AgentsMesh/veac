use std::path::Path;

use super::BackendCommand;

const FILTER_COMPLEX_THREADS: &str = "1";
const FILTER_INPUT_THREADS: &str = "1";
const FILTER_OUTPUT_THREADS: &str = "1";

impl BackendCommand {
    pub fn to_args(&self) -> Vec<String> {
        self.build_args(None)
    }

    pub fn to_args_with_filter_script(&self, path: &Path) -> Vec<String> {
        self.build_args(Some(path))
    }

    fn build_args(&self, script: Option<&Path>) -> Vec<String> {
        let bounded_graph =
            self.filter_graph.is_some() && (script.is_some() || self.has_alpha_prores_output());
        let mut args = vec![
            "-y".to_owned(),
            "-hide_banner".to_owned(),
            "-nostdin".to_owned(),
        ];
        for input in &self.inputs {
            if bounded_graph {
                args.extend(["-threads".to_owned(), FILTER_INPUT_THREADS.to_owned()]);
            }
            args.extend([
                "-noautorotate".to_owned(),
                "-i".to_owned(),
                input.path.to_string_lossy().into_owned(),
            ]);
        }
        self.append_filter_arguments(&mut args, script, bounded_graph);
        for map in &self.maps {
            args.extend(["-map".to_owned(), map.clone()]);
        }
        args.extend(self.output_args.clone());
        if bounded_graph {
            args.extend(["-threads".to_owned(), FILTER_OUTPUT_THREADS.to_owned()]);
        }
        args.push(self.output_path.to_string_lossy().into_owned());
        args
    }

    fn append_filter_arguments(
        &self,
        args: &mut Vec<String>,
        script: Option<&Path>,
        bounded_graph: bool,
    ) {
        let Some(graph) = &self.filter_graph else {
            return;
        };
        if bounded_graph {
            args.extend([
                "-filter_complex_threads".to_owned(),
                FILTER_COMPLEX_THREADS.to_owned(),
            ]);
        }
        match script {
            Some(path) => args.extend([
                "-filter_complex_script".to_owned(),
                path.to_string_lossy().into_owned(),
            ]),
            None => args.extend(["-filter_complex".to_owned(), graph.clone()]),
        }
    }

    fn has_alpha_prores_output(&self) -> bool {
        option_is(&self.output_args, "-c:v", "prores_ks")
            && option_starts_with(&self.output_args, "-pix_fmt", "yuva444p")
    }
}

fn option_is(arguments: &[String], name: &str, expected: &str) -> bool {
    arguments
        .windows(2)
        .any(|pair| pair[0] == name && pair[1] == expected)
}

fn option_starts_with(arguments: &[String], name: &str, prefix: &str) -> bool {
    arguments
        .windows(2)
        .any(|pair| pair[0] == name && pair[1].starts_with(prefix))
}
