mod alignment;
mod fonts;
mod layout;

use std::path::PathBuf;

use veac_plan::canonical::*;
use veac_plan::{PlanInputId, ResolvedClipSource, ResolvedText};

use super::support::{
    ass_script, bindings, emit_video_command, resolved, set_input_identity, text_fixture,
};

fn text_content(plan: &mut veac_plan::ResolvedRenderPlan) -> &mut ResolvedText {
    match &mut plan.sequences[0].tracks[1].clips[0].source {
        ResolvedClipSource::Text { content } => content,
        _ => panic!("text fixture"),
    }
}

fn point(x: f64, y: f64) -> Point {
    Point {
        x: pixels(x),
        y: pixels(y),
    }
}

fn pixels(value: f64) -> Length {
    Length {
        value,
        unit: LengthUnit::Pixels,
    }
}

fn cjk_font_path() -> PathBuf {
    [
        "/System/Library/Fonts/STHeiti Medium.ttc",
        "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.otf",
    ]
    .iter()
    .map(PathBuf::from)
    .find(|path| path.is_file())
    .expect("CI installs a CJK test font")
}

fn small_script_font() -> PathBuf {
    [
        "/System/Library/Fonts/Supplemental/NotoSansHatran-Regular.ttf",
        "/usr/share/fonts/truetype/noto/NotoSansHatran-Regular.ttf",
    ]
    .iter()
    .map(PathBuf::from)
    .find(|path| path.is_file())
    .expect("CI installs a small Noto script font")
}

fn add_font_input(plan: &mut veac_plan::ResolvedRenderPlan, id: &str) -> PlanInputId {
    let primary = text_content(plan).styled_mut().unwrap().font.clone();
    let id = PlanInputId::new(id).unwrap();
    let mut input = plan
        .inputs
        .iter()
        .find(|input| input.id == primary.input_id)
        .unwrap()
        .clone();
    input.id = id.clone();
    plan.inputs.push(input);
    plan.inputs.sort_by(|left, right| left.id.cmp(&right.id));
    id
}
