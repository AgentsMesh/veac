/// Variable, expression, and asset resolution helpers.
use std::collections::HashMap;

use crate::ast::*;
use crate::error::{ErrorKind, VeacError};
use crate::ir::*;

use super::SemanticAnalyzer;

impl SemanticAnalyzer<'_> {
    pub(crate) fn resolve_variables(
        &self,
        vars: &[LetDecl],
    ) -> Result<HashMap<String, Expression>, VeacError> {
        let mut map = HashMap::new();
        for v in vars {
            if map.contains_key(&v.name) {
                return Err(VeacError::new(
                    ErrorKind::DuplicateDefinition,
                    format!("variable `{}` is already defined", v.name),
                    None,
                ));
            }
            map.insert(v.name.clone(), v.value.clone());
        }
        Ok(map)
    }

    pub(crate) fn resolve_expression<'b>(
        &self,
        expr: &'b Expression,
        variables: &'b HashMap<String, Expression>,
    ) -> Result<&'b Expression, VeacError> {
        match expr {
            Expression::Ident(name) => variables.get(name.as_str()).ok_or_else(|| {
                VeacError::new(
                    ErrorKind::UndefinedVariable,
                    format!("undefined variable `{name}`"),
                    None,
                )
                .with_hint(format!("define it with `let {name} = ...`"))
            }),
            other => Ok(other),
        }
    }

    pub(crate) fn resolve_assets(&self, decls: &[AssetDecl]) -> Result<Vec<IrAsset>, VeacError> {
        let mut names = HashMap::new();
        let mut assets = Vec::new();

        for decl in decls {
            if names.insert(&decl.name, ()).is_some() {
                return Err(VeacError::new(
                    ErrorKind::DuplicateDefinition,
                    format!("duplicate asset name `{}`", decl.name),
                    None,
                ));
            }

            let resolved_path = self.base_dir.join(&decl.path);
            assets.push(IrAsset {
                name: decl.name.clone(),
                kind: match decl.kind {
                    AssetKind::Video => IrAssetKind::Video,
                    AssetKind::Audio => IrAssetKind::Audio,
                    AssetKind::Image => IrAssetKind::Image,
                },
                path: resolved_path,
                media_info: None,
            });
        }

        Ok(assets)
    }
}
