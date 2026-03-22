/// Resolution of output declarations for multi-output support.
use std::collections::HashMap;

use crate::ast::*;
use crate::error::VeacError;
use crate::ir::*;

use super::SemanticAnalyzer;

impl SemanticAnalyzer<'_> {
    pub(crate) fn resolve_outputs(
        &self,
        outputs: &[OutputDecl],
        variables: &HashMap<String, Expression>,
    ) -> Result<Vec<IrOutputConfig>, VeacError> {
        let mut configs = Vec::new();
        for out in outputs {
            let mut width = None;
            let mut height = None;
            let mut format = None;
            let mut codec = None;
            let mut quality = None;

            for attr in &out.attributes {
                let val = self.resolve_expression(&attr.value, variables)?;
                match attr.key.as_str() {
                    "resolution" => {
                        let (w, h) = Self::parse_resolution(val)?;
                        width = Some(w);
                        height = Some(h);
                    }
                    "format" => format = Some(Self::parse_format(val)?),
                    "codec" => codec = Some(Self::parse_codec(val)?),
                    "quality" => quality = Some(Self::parse_quality(val)?),
                    _ => {}
                }
            }

            configs.push(IrOutputConfig {
                path: self.base_dir.join(&out.path),
                width,
                height,
                format,
                codec,
                quality,
            });
        }
        Ok(configs)
    }
}
