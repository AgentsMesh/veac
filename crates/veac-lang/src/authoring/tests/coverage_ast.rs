use crate::authoring::{
    AnnotationTargetDecl, AnnotationTimingDecl, AudioModifierDecl, ColorModifierDecl,
    CompositeModifierDecl, EffectModifierDecl, Identifier, LayoutModifierDecl, MaskModifierDecl,
    MaskShapeDecl, ModifierDecl, NumberLiteral, Span, Spanned, SurfaceModifierDecl,
    TransformModifierDecl,
};

fn id(value: &str, span: Span) -> Identifier {
    Spanned {
        value: value.to_owned(),
        span,
    }
}

#[test]
fn annotation_nodes_report_their_own_source_spans() {
    let spans = [
        AnnotationTargetDecl::Project {
            span: Span { start: 1, end: 2 },
        },
        AnnotationTargetDecl::Sequence(id("sequence", Span { start: 2, end: 3 })),
        AnnotationTargetDecl::Layer(id("layer", Span { start: 3, end: 4 })),
        AnnotationTargetDecl::Item(id("item", Span { start: 4, end: 5 })),
        AnnotationTargetDecl::Resource(id("resource", Span { start: 5, end: 6 })),
        AnnotationTargetDecl::Multicam(id("multicam", Span { start: 6, end: 7 })),
    ]
    .map(|value| value.span());
    assert_eq!(spans.map(|span| span.start), [1, 2, 3, 4, 5, 6]);

    let literal = crate::authoring::NumberLiteral {
        raw: "1s".into(),
        span: Span { start: 8, end: 9 },
    };
    let timings = [
        AnnotationTimingDecl::Untimed {
            span: Span { start: 10, end: 11 },
        },
        AnnotationTimingDecl::Point {
            at: literal.clone(),
            span: Span { start: 11, end: 12 },
        },
        AnnotationTimingDecl::Range {
            at: literal.clone(),
            duration: literal,
            span: Span { start: 12, end: 13 },
        },
    ];
    assert_eq!(timings.map(|value| value.span().start), [10, 11, 12]);
}

#[test]
fn every_modifier_exposes_its_identifier_and_complete_span() {
    let canonical_space = veac_ir::ColorSpace {
        primaries: veac_ir::ColorPrimaries::Bt709,
        transfer: veac_ir::ColorTransfer::Bt709,
        matrix: veac_ir::ColorMatrix::Bt709,
        range: veac_ir::ColorRange::Limited,
    };
    let make_id = |value: &str| id(value, Span { start: 1, end: 2 });
    let spans = (10..18).map(|start| Span {
        start,
        end: start + 1,
    });
    let mut spans = spans.into_iter();
    let modifiers = vec![
        ModifierDecl::Layout(LayoutModifierDecl {
            id: make_id("layout"),
            placement: None,
            frame: None,
            span: spans.next().unwrap(),
        }),
        ModifierDecl::Transform(TransformModifierDecl {
            id: make_id("transform"),
            position: None,
            scale: None,
            shear: None,
            rotation: None,
            anchor: None,
            crop: None,
            flip_horizontal: None,
            flip_vertical: None,
            span: spans.next().unwrap(),
        }),
        ModifierDecl::Composite(CompositeModifierDecl {
            id: make_id("composite"),
            opacity: None,
            z_index: None,
            blend: None,
            span: spans.next().unwrap(),
        }),
        ModifierDecl::Surface(SurfaceModifierDecl {
            id: make_id("surface"),
            corner_radius: NumberLiteral {
                raw: "12px".into(),
                span: Span::default(),
            },
            shadow: None,
            span: spans.next().unwrap(),
        }),
        ModifierDecl::Mask(MaskModifierDecl {
            id: make_id("mask"),
            shape: MaskShapeDecl::Linear,
            position: None,
            scale: None,
            rotation: None,
            feather: None,
            expansion: None,
            invert: None,
            span: spans.next().unwrap(),
        }),
        ModifierDecl::Audio(AudioModifierDecl {
            id: make_id("audio"),
            gain: None,
            pan: None,
            muted: None,
            normalize: None,
            pitch: None,
            processors: Vec::new(),
            crossfade: None,
            span: spans.next().unwrap(),
        }),
        ModifierDecl::Color(ColorModifierDecl {
            id: make_id("color"),
            input: Spanned {
                value: canonical_space,
                span: Span::default(),
            },
            working: Spanned {
                value: canonical_space,
                span: Span::default(),
            },
            output: Spanned {
                value: canonical_space,
                span: Span::default(),
            },
            stages: Vec::new(),
            span: spans.next().unwrap(),
        }),
        ModifierDecl::Effect(EffectModifierDecl {
            id: make_id("effect"),
            effect_type: make_id("video.blur"),
            enabled: None,
            record: None,
            parameters: Vec::new(),
            span: spans.next().unwrap(),
        }),
    ];
    let names = modifiers
        .iter()
        .map(|value| value.id().value.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        names,
        [
            "layout",
            "transform",
            "composite",
            "surface",
            "mask",
            "audio",
            "color",
            "effect"
        ]
    );
    assert_eq!(
        modifiers.iter().map(ModifierDecl::span).collect::<Vec<_>>(),
        (10..18)
            .map(|start| Span {
                start,
                end: start + 1
            })
            .collect::<Vec<_>>()
    );
}

#[test]
fn authored_output_defaults_are_canonical_baselines() {
    let space = crate::authoring::ColorSpace::default();
    assert_eq!(space.primaries, crate::authoring::ColorPrimaries::Bt709);
    assert_eq!(space.transfer, crate::authoring::ColorTransfer::Bt709);
    assert_eq!(space.matrix, crate::authoring::ColorMatrix::Bt709);
    assert_eq!(space.range, crate::authoring::ColorRange::Limited);
}
