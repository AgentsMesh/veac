use crate::authoring::{
    Diagnostic, MatteEndpoints, MatteMode, MatteRelation, MatteStyle, SemanticBlock, Spanned,
};

use super::{relation_endpoint, relation_style, relation_value, Parser};

pub(super) fn parse(parser: &mut Parser, mut body: SemanticBlock) -> Option<MatteRelation> {
    let endpoints =
        super::relation::required_block(parser, &mut body, "endpoints", "matte relation")
            .and_then(|block| endpoints(parser, block));
    let style = super::relation::required_block(parser, &mut body, "style", "matte relation")
        .and_then(|block| style(parser, block));
    relation_value::finish(parser, body, "matte relation");
    Some(MatteRelation {
        endpoints: endpoints?,
        style: style?,
    })
}

fn endpoints(parser: &mut Parser, mut body: SemanticBlock) -> Option<MatteEndpoints> {
    let producer =
        relation_value::take_entry(parser, &mut body, "producer", true, "matte endpoints")
            .and_then(|entry| relation_endpoint::item(parser, entry, "matte endpoints.producer"));
    let consumer =
        relation_value::take_entry(parser, &mut body, "consumer", true, "matte endpoints")
            .and_then(|entry| relation_endpoint::item(parser, entry, "matte endpoints.consumer"));
    relation_value::finish(parser, body, "matte endpoints");
    Some(MatteEndpoints {
        producer: producer?,
        consumer: consumer?,
    })
}

fn style(parser: &mut Parser, body: SemanticBlock) -> Option<MatteStyle> {
    if body.entries.len() != 1 {
        parser.diagnostic(Diagnostic::new(
            "AUTHORING_RELATION_STYLE_SHAPE",
            "matte style requires exactly one alpha or luma variant",
            body.span,
        ));
        return None;
    }
    let entry = body.entries.into_iter().next()?;
    let mode = match entry.name.value.as_str() {
        "alpha" => MatteMode::Alpha,
        "luma" => MatteMode::Luma,
        _ => {
            relation_value::unknown(
                parser,
                "AUTHORING_UNKNOWN_MATTE_MODE",
                "matte mode",
                entry.name,
            );
            return None;
        }
    };
    let mode = Spanned {
        value: mode,
        span: entry.name.span,
    };
    let mut body = relation_style::variant_body(parser, entry, "matte style")?;
    let invert = relation_value::take_entry(parser, &mut body, "invert", true, "matte style")
        .and_then(|entry| relation_value::boolean(parser, entry, "matte style.invert"));
    relation_value::finish(parser, body, "matte style");
    Some(MatteStyle {
        mode,
        invert: invert?,
    })
}
