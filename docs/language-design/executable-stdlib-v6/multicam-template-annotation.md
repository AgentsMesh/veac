# Multicam, Templates, And Annotations

## DomainTypes

```text
MulticamGroup, MulticamAngle, MulticamSwitch, MulticamSync, MulticamSyncBasis
TemplateContract, SlotKind, FillMode, SourceDurationPolicy, TemplateTextPolicy
Annotation, AnnotationTarget, AnnotationSpan, AnnotationPayload
AnnotationProvenance, AnnotationProvenanceChoice, MarkerColor
LanguageConfidence, FillerSuggestion, ReviewAction
```

## Multicam Values

```veac
multicam_sync_timecode() -> MulticamSyncBasis
multicam_sync_audio() -> MulticamSyncBasis
multicam_sync_manual() -> MulticamSyncBasis
multicam_angle(key: identifier, resource: Resource,
               source_offset: time) -> MulticamAngle
multicam_sync(basis: MulticamSyncBasis, reference: MulticamAngle) -> MulticamSync
multicam_switch(angle: MulticamAngle, range: TimeRange) -> MulticamSwitch
```

Angles accept video Resources and are unique by key. Source offset maps group-local zero to source
time. Switch ranges are clip-local, ordered, contiguous, and form a complete partition.

## Multicam Graph Operations

```veac
multicam_group(key: identifier, sync: MulticamSync,
               angles: list<MulticamAngle>) -> MulticamGroup
Project.with_multicam_group(group: MulticamGroup) -> Project
source_multicam(group: MulticamGroup, switches: list<MulticamSwitch>) -> Source
```

The group constructor and owner method are `GraphEmit`; source construction is Pure. A group has at
least two angles, contains its reference angle, and belongs to exactly one Project. The source
signature is the operation cataloged in `sources-generators.md`.

## Template Contract

```veac
slot_video() -> SlotKind
slot_image() -> SlotKind
slot_video_or_image() -> SlotKind
slot_text() -> SlotKind
fill_fit_duration() -> FillMode
fill_take_head() -> FillMode
fill_take_center() -> FillMode
source_duration_any() -> SourceDurationPolicy
source_duration_at_least(value: time) -> SourceDurationPolicy
template_text_locked() -> TemplateTextPolicy
template_text_editable() -> TemplateTextPolicy
template_contract(kind: SlotKind, fill: FillMode, label: text,
                  duration: SourceDurationPolicy,
                  text: TemplateTextPolicy) -> TemplateContract
Item.with_template(contract: TemplateContract) -> Item
```

`Item.with_template` is `GraphEmit`. Label is non-empty and at most 256 UTF-8 bytes. Media slots own
one media placeholder, require authored visual state, and must use `template_text_locked`. Text slots
own a text placeholder on a visual layer, use `fill_fit_duration` with no source-duration minimum,
and may be locked or editable. Text editability is therefore part of one typed slot contract rather
than an unrelated Item flag.

## Annotation Targets And Spans

```veac
annotation_target_project() -> AnnotationTarget
annotation_target_sequence(sequence: Sequence) -> AnnotationTarget
annotation_target_layer(layer: Layer) -> AnnotationTarget
annotation_target_item(item: Item) -> AnnotationTarget
annotation_target_resource(resource: Resource) -> AnnotationTarget
annotation_target_multicam(group: MulticamGroup) -> AnnotationTarget
annotation_untimed() -> AnnotationSpan
annotation_point(at: time) -> AnnotationSpan
annotation_range(range: TimeRange) -> AnnotationSpan
```

Target handles must be reachable from the owner Project. Point and range times use the target's
timeline domain; untimed annotations carry no manufactured zero timestamp.

## Annotation Provenance

```veac
annotation_provenance(producer: text, request: ContentIdentity,
                      response: ContentIdentity) -> AnnotationProvenance
annotation_provenance_none() -> AnnotationProvenanceChoice
annotation_provenance_present(value: AnnotationProvenance) -> AnnotationProvenanceChoice
marker_color_none() -> MarkerColor
marker_color_present(value: color) -> MarkerColor
```

Producer is a stable non-empty tool/model identity. Both digests are SHA-256 values, not arbitrary
text. Provenance absence is explicit for human-authored annotations.

## Closed Payloads

```veac
language_confidence(language: text, confidence: percent) -> LanguageConfidence
filler_keep() -> FillerSuggestion
filler_delete() -> FillerSuggestion
filler_tighten() -> FillerSuggestion
review_keep() -> ReviewAction
review_remove() -> ReviewAction
annotation_marker(label: text, marker_color: MarkerColor) -> AnnotationPayload
annotation_language(scores: list<LanguageConfidence>) -> AnnotationPayload
annotation_scene_boundary(confidence: percent, hard_cut: bool) -> AnnotationPayload
annotation_scene() -> AnnotationPayload
annotation_beat(confidence: percent, bar: int, beat_in_bar: int,
                tempo_bpm: scalar, meter: int) -> AnnotationPayload
annotation_silence(mean_db: scalar, confidence: percent) -> AnnotationPayload
annotation_filler(token: text, confidence: percent,
                  suggestion: FillerSuggestion) -> AnnotationPayload
annotation_highlight(score: percent, rationale: text,
                     evidence: list<text>) -> AnnotationPayload
annotation_review(action: ReviewAction, rationale: text,
                  confidence: percent) -> AnnotationPayload
```

Language scores are non-empty with unique language tags. Bar, beat, tempo, and meter are positive.
Rationale and evidence preserve authored order. Review action is the closed keep/remove set even though
the current canonical storage field is text; arbitrary review action strings never enter Surface.

## Annotation Graph Operations

```veac
annotation(key: identifier, target: AnnotationTarget, span: AnnotationSpan,
           payload: AnnotationPayload,
           provenance: AnnotationProvenanceChoice) -> Annotation
Project.with_annotation(annotation: Annotation) -> Project
```

Both are `GraphEmit`. Annotation keys are unique in Project scope. Span/payload rules are validated
before attachment, and failure rolls back the entire graph transaction.
