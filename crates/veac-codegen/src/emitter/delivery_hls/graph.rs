use veac_plan::canonical::HlsPackage;
use veac_plan::ResolvedSequence;

use super::super::{audio, color_space, hls_encoding, sequence, CodegenErrors, EmitContext};

pub(super) struct Outputs {
    pub video: Vec<String>,
    pub audio: Option<Vec<String>>,
}

pub(super) fn build(
    context: &mut EmitContext<'_>,
    sequence: &ResolvedSequence,
    settings: &HlsPackage,
) -> Result<Outputs, CodegenErrors> {
    let visual = sequence::build_entry(context, sequence)?;
    let visual = sequence::conform_output(context, visual, sequence);
    let sources = split(context, &visual, settings.renditions.len(), false);
    let video = sources
        .into_iter()
        .zip(&settings.renditions)
        .map(|(source, rendition)| {
            let scaled = context.graph.filter(
                &[&source],
                format!(
                    "scale={}:{}:flags=lanczos,setsar=1",
                    rendition.raster.width, rendition.raster.height
                ),
                "hlsv",
            );
            match hls_encoding::video(&rendition.encoding).color_space {
                Some(space) => color_space::tag(context, scaled, space),
                None => scaled,
            }
        })
        .collect();
    let audio = settings
        .audio
        .as_ref()
        .map(|output| {
            let spec = hls_encoding::audio(&output.encoding);
            audio::build_stem(context, sequence, &output.source, spec)
        })
        .transpose()?
        .map(|source| split(context, &source, settings.renditions.len(), true));
    Ok(Outputs { video, audio })
}

fn split(context: &mut EmitContext<'_>, input: &str, count: usize, audio: bool) -> Vec<String> {
    if count == 1 {
        return vec![input.to_owned()];
    }
    context.graph.filter_many(
        &[input],
        format!("{}split={count}", if audio { "a" } else { "" }),
        if audio { "hlsa" } else { "hlssplitv" },
        count,
    )
}
