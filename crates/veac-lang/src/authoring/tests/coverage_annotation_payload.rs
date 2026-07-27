use crate::authoring::{format_document, lower_document, parse};

use super::{coverage_annotation_support::all_annotations, project};

#[test]
fn all_annotation_payload_branches_lower_to_valid_typed_ir() {
    let body = format!(
        r#"resource image poster {{ locator local {{ path "poster.png"; }} }}
  sequence main {{
    layer visual canvas {{
      item hero {{
        source generated transparent;
        record {{ at 0s; duration 4s; }}
      }}
    }}
  }}
  {}"#,
        all_annotations()
    );
    let source = project(&body);
    let document = parse(&source).unwrap_or_else(|error| {
        let span = error.as_slice()[0].span;
        let start = span.start.saturating_sub(80);
        let end = (span.end + 80).min(source.len());
        panic!("{error}; offending input: {:?}", &source[start..end])
    });
    let formatted = format_document(&document);
    assert_eq!(format_document(&parse(&formatted).unwrap()), formatted);
    assert!(formatted.contains("suggestion delete;"));
    assert!(formatted.contains("action remove;"));

    let envelope = lower_document(&document).unwrap_or_else(|error| panic!("{error}"));
    let values = &envelope.project.annotations;
    assert_eq!(values.len(), 13);
    assert!(matches!(
        values[0].target,
        veac_ir::AnnotationTarget::Project
    ));
    assert!(matches!(
        values[1].target,
        veac_ir::AnnotationTarget::Sequence { .. }
    ));
    assert!(matches!(
        values[2].target,
        veac_ir::AnnotationTarget::Track { .. }
    ));
    assert!(matches!(
        values[3].target,
        veac_ir::AnnotationTarget::Clip { .. }
    ));
    assert!(matches!(
        values[12].target,
        veac_ir::AnnotationTarget::Material { .. }
    ));

    let veac_ir::AnnotationPayload::Marker { color, .. } = &values[0].payload else {
        panic!("marker expected")
    };
    assert!(color.is_some());
    let veac_ir::AnnotationPayload::Language { scores } = &values[1].payload else {
        panic!("language expected")
    };
    assert_eq!(
        scores
            .iter()
            .map(|value| value.language.as_str())
            .collect::<Vec<_>>(),
        ["fr", "en"]
    );
    assert_eq!(
        scores
            .iter()
            .map(|value| value.confidence)
            .collect::<Vec<_>>(),
        [0.9, 0.6]
    );
    assert!(matches!(values[1].span, veac_ir::AnnotationSpan::Untimed));
    assert!(matches!(
        values[2].span,
        veac_ir::AnnotationSpan::Point { .. }
    ));
    assert!(matches!(
        values[3].span,
        veac_ir::AnnotationSpan::Range { .. }
    ));

    let veac_ir::AnnotationPayload::Beat {
        bar,
        beat_in_bar,
        meter,
        tempo,
        ..
    } = values[4].payload
    else {
        panic!("beat expected")
    };
    assert_eq!((bar, beat_in_bar, meter), (7, 2, 4));
    assert_eq!((tempo.numerator, tempo.denominator), (241, 2));
    let suggestions = values[6..9]
        .iter()
        .map(|value| match value.payload {
            veac_ir::AnnotationPayload::Filler { suggestion, .. } => suggestion,
            _ => panic!("filler expected"),
        })
        .collect::<Vec<_>>();
    assert_eq!(
        suggestions,
        [
            veac_ir::FillerSuggestion::Keep,
            veac_ir::FillerSuggestion::Delete,
            veac_ir::FillerSuggestion::Tighten,
        ]
    );
    let veac_ir::AnnotationPayload::Highlight { evidence, .. } = &values[9].payload else {
        panic!("highlight expected")
    };
    assert_eq!(evidence, &["a-first".to_owned(), "z-last".to_owned()]);
    for (index, expected) in [(10, "keep"), (11, "remove")] {
        let veac_ir::AnnotationPayload::Review { action, .. } = &values[index].payload else {
            panic!("review expected")
        };
        assert_eq!(action, expected);
    }
    veac_ir::validate(&envelope).unwrap();
}
