/// FFmpeg output arguments and command serialization.
use super::FfmpegCommand;

pub fn build_output_args(project: &veac_lang::ir::IrProject) -> Vec<String> {
    let mut args = Vec::new();
    args.push("-c:v".into());
    args.push(project.codec.ffmpeg_encoder().into());
    args.push("-preset".into());
    args.push(project.quality.ffmpeg_preset().into());
    args.push("-crf".into());
    args.push(project.quality.ffmpeg_crf().to_string());
    args.push("-r".into());
    args.push(project.fps.to_string());
    args.push("-s".into());
    args.push(format!("{}x{}", project.width, project.height));
    args
}

impl FfmpegCommand {
    /// Produce the full argument list for `std::process::Command`.
    pub fn to_args(&self) -> Vec<String> {
        let mut args = Vec::new();
        args.push("-y".into());

        for input in &self.inputs {
            args.push("-i".into());
            args.push(input.path.to_string_lossy().into_owned());
        }

        if let Some(ref fg) = self.filter_graph {
            args.push("-filter_complex".into());
            args.push(fg.clone());
        }

        for m in &self.map_args {
            args.push("-map".into());
            args.push(m.clone());
        }

        args.extend(self.output_args.iter().cloned());
        args.push(self.output_path.to_string_lossy().into_owned());
        args
    }

    /// Produce a human-readable command string for `veac plan`.
    pub fn to_command_string(&self) -> String {
        let mut parts = vec!["ffmpeg".to_string()];
        parts.extend(self.to_args());
        parts.join(" ")
    }
}
