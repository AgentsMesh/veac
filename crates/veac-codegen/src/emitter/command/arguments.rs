use std::path::Path;

use super::BackendCommand;

impl BackendCommand {
    pub fn to_args(&self) -> Vec<String> {
        self.build_args(None)
    }

    pub fn to_args_with_filter_script(&self, path: &Path) -> Vec<String> {
        self.build_args(Some(path))
    }

    fn build_args(&self, script: Option<&Path>) -> Vec<String> {
        let mut args = vec![
            "-y".to_owned(),
            "-hide_banner".to_owned(),
            "-nostdin".to_owned(),
        ];
        for input in &self.inputs {
            args.extend([
                "-noautorotate".to_owned(),
                "-i".to_owned(),
                input.path.to_string_lossy().into_owned(),
            ]);
        }
        self.append_filter_arguments(&mut args, script);
        for map in &self.maps {
            args.extend(["-map".to_owned(), map.clone()]);
        }
        args.extend(self.output_args.clone());
        args.push(self.output_path.to_string_lossy().into_owned());
        args
    }

    fn append_filter_arguments(&self, args: &mut Vec<String>, script: Option<&Path>) {
        let Some(graph) = &self.filter_graph else {
            return;
        };
        match script {
            Some(path) => args.extend([
                "-filter_complex_script".to_owned(),
                path.to_string_lossy().into_owned(),
            ]),
            None => args.extend(["-filter_complex".to_owned(), graph.clone()]),
        }
    }
}
