use std::collections::BTreeMap;
use std::sync::Arc;

use crate::authoring::Span;

use super::expression::{Value, ValueType};

mod interface;
pub(crate) use interface::ComponentInterface;
mod preset_map;
pub(crate) use preset_map::{preset_key, preset_name_exists, PresetKey, PresetMap};

#[derive(Debug, Clone)]
pub(crate) struct SurfaceFile {
    pub path: String,
    pub source: String,
    pub kind: FileKind,
    pub imports: Vec<ImportDecl>,
    pub constants: Vec<ConstDecl>,
    pub presets: Vec<PresetDecl>,
    pub components: Vec<ComponentDecl>,
    pub instances: Vec<InstanceDecl>,
    pub project: Option<ProjectDecl>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum FileKind {
    Entry,
    Module,
}

#[derive(Debug, Clone)]
pub(crate) struct ImportDecl {
    pub path: String,
    pub alias: String,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub(crate) struct ConstDecl {
    pub name: String,
    pub value_type: ValueType,
    pub expression: String,
    pub expression_span: Span,
    pub exported: bool,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum PresetKind {
    TextStyle,
    TextLayout,
    ModifierStack,
    EffectPipeline,
    ColorPipeline,
    AudioProcessors,
    DeliveryProfile,
}

#[derive(Debug, Clone)]
pub(crate) struct PresetDecl {
    pub kind: PresetKind,
    pub name: String,
    pub body: RawBlock,
    pub exported: bool,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub(crate) struct ComponentDecl {
    pub name: String,
    pub parameters: Vec<ParameterDecl>,
    pub slots: Vec<SlotDecl>,
    pub instances: Vec<InstanceDecl>,
    pub body: RawBlock,
    pub exported: bool,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub(crate) struct ParameterDecl {
    pub name: String,
    pub value_type: ValueType,
    pub default: Option<ExpressionBinding>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SlotKind {
    Video,
    Audio,
    Visual,
    Text,
    Caption,
    Sequence,
}

#[derive(Debug, Clone)]
pub(crate) struct SlotDecl {
    pub name: String,
    pub kind: SlotKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub(crate) struct InstanceDecl {
    pub component: String,
    pub id: String,
    pub bindings: BTreeMap<String, ExpressionBinding>,
    pub fills: BTreeMap<String, RawBlock>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub(crate) struct ExpressionBinding {
    pub source: String,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub(crate) struct RawBlock {
    pub content_span: Span,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub(crate) struct ProjectDecl {
    pub name: String,
    pub name_span: Span,
    pub body: RawBlock,
    pub span: Span,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct Scope {
    pub values: Arc<ValueMap>,
    pub presets: Arc<PresetMap>,
    pub components: BTreeMap<String, ComponentKey>,
}

pub(crate) type ValueMap = BTreeMap<String, Arc<Value>>;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct ComponentKey {
    pub path: String,
    pub name: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ResolvedComponent {
    pub declaration: ComponentDecl,
    pub interface: ComponentInterface,
    pub source_path: String,
    pub source: Arc<str>,
    pub captured: Arc<CapturedScope>,
}

#[derive(Debug)]
pub(crate) struct CapturedScope {
    pub values: Arc<ValueMap>,
    pub presets: Arc<PresetMap>,
    pub components: BTreeMap<String, ComponentKey>,
}

pub(crate) type ComponentCatalog = BTreeMap<ComponentKey, Arc<ResolvedComponent>>;
