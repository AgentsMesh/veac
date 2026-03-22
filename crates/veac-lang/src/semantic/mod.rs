mod resolve;
mod resolve_assets;
mod resolve_clip;
mod resolve_image;
mod resolve_output;
mod resolve_text;
mod resolve_timeline_items;
mod resolve_transition;

use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::ast::*;
use crate::error::{ErrorKind, VeacError};
use crate::ir::*;
use crate::lexer::Lexer;
use crate::parser::Parser;

/// Semantic analyzer: validates the AST and produces a fully resolved IR.
pub struct SemanticAnalyzer<'a> {
    pub(crate) base_dir: &'a Path,
}

impl<'a> SemanticAnalyzer<'a> {
    pub fn new(base_dir: &'a Path) -> Self {
        Self { base_dir }
    }

    /// Analyze a parsed program and produce validated IR.
    pub fn analyze(&self, program: &Program) -> Result<IrProgram, VeacError> {
        let mut merged = program.clone();
        let mut visited = HashSet::new();
        self.process_includes(&mut merged, &mut visited)?;

        let project_decl = merged.project.as_ref().ok_or_else(|| {
            VeacError::new(ErrorKind::NoProject, "missing `project` declaration", None)
                .with_hint("add a `project \"name\" { ... }` block at the top of the file")
        })?;

        if merged.timelines.is_empty() {
            return Err(VeacError::new(
                ErrorKind::NoTimeline,
                "no `timeline` declaration found",
                None,
            )
            .with_hint("add a `timeline main { ... }` block"));
        }

        let variables = self.resolve_variables(&merged.variables)?;
        let project = self.resolve_project(project_decl, &variables)?;
        let assets = self.resolve_assets(&merged.assets)?;
        let asset_map: HashMap<&str, &IrAsset> =
            assets.iter().map(|a| (a.name.as_str(), a)).collect();
        let timeline =
            self.resolve_timeline(&merged.timelines[0], &asset_map, &variables, project.fps)?;
        let outputs = self.resolve_outputs(&merged.outputs, &variables)?;

        Ok(IrProgram {
            project,
            assets,
            timeline,
            outputs,
        })
    }

    /// Recursively process include declarations, merging their content.
    fn process_includes(
        &self,
        program: &mut Program,
        visited: &mut HashSet<std::path::PathBuf>,
    ) -> Result<(), VeacError> {
        let includes: Vec<IncludeDecl> = program.includes.drain(..).collect();

        for inc in includes {
            let resolved_path = self.base_dir.join(&inc.path).canonicalize().map_err(|_| {
                VeacError::new(
                    ErrorKind::AssetFileNotFound,
                    format!("include file not found: {}", inc.path.display()),
                    None,
                )
            })?;

            if !visited.insert(resolved_path.clone()) {
                continue;
            }

            let source = std::fs::read_to_string(&resolved_path).map_err(|_| {
                VeacError::new(
                    ErrorKind::AssetFileNotFound,
                    format!("cannot read include file: {}", resolved_path.display()),
                    None,
                )
            })?;

            let mut lexer = Lexer::new(&source);
            let tokens = lexer.tokenize()?;
            let mut parser = Parser::new(tokens);
            let mut included = parser.parse()?;

            let inc_dir = resolved_path.parent().unwrap_or(Path::new("."));
            let sub_analyzer = SemanticAnalyzer::new(inc_dir);
            sub_analyzer.process_includes(&mut included, visited)?;

            program.assets.extend(included.assets);
            program.variables.extend(included.variables);
            program.timelines.extend(included.timelines);
        }

        Ok(())
    }

    fn resolve_timeline(
        &self,
        decl: &TimelineDecl,
        assets: &HashMap<&str, &IrAsset>,
        variables: &HashMap<String, Expression>,
        fps: u32,
    ) -> Result<IrTimeline, VeacError> {
        let mut tracks = Vec::new();

        for track_decl in &decl.tracks {
            let kind = match track_decl.kind {
                TrackKind::Video => IrTrackKind::Video,
                TrackKind::Audio => IrTrackKind::Audio,
                TrackKind::Text => IrTrackKind::Text,
                TrackKind::Overlay => IrTrackKind::Overlay,
            };

            let mut items = Vec::new();
            for item in &track_decl.items {
                match item {
                    TrackItem::Clip(c) => items.push(IrTrackItem::Clip(
                        self.resolve_clip(c, assets, variables, fps)?,
                    )),
                    TrackItem::TextOverlay(t) => items.push(IrTrackItem::TextOverlay(
                        self.resolve_text_overlay(t, variables, fps)?,
                    )),
                    TrackItem::Transition(t) => items.push(IrTrackItem::Transition(
                        self.resolve_transition(t, variables, fps)?,
                    )),
                    TrackItem::ImageOverlay(i) => items.push(IrTrackItem::ImageOverlay(
                        self.resolve_image_overlay(i, assets, variables, fps)?,
                    )),
                    TrackItem::Gap(g) => {
                        items.push(IrTrackItem::Gap(self.resolve_gap(g, variables, fps)?))
                    }
                    TrackItem::Freeze(f) => items.push(IrTrackItem::Freeze(
                        self.resolve_freeze(f, assets, variables, fps)?,
                    )),
                    TrackItem::Pip(p) => items.push(IrTrackItem::Pip(
                        self.resolve_pip(p, assets, variables, fps)?,
                    )),
                    TrackItem::Subtitle(s) => items.push(IrTrackItem::Subtitle(IrSubtitle {
                        path: self.base_dir.join(&s.path),
                    })),
                }
            }

            tracks.push(IrTrack { kind, items });
        }

        Ok(IrTimeline {
            name: decl.name.clone(),
            tracks,
        })
    }
}
