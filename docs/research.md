# Research Notes

This file records external systems that influenced VEAC. They are architectural references, not compatibility targets or bundled dependencies.

## Declarative Editing Models

| Source | Useful idea | VEAC consequence |
| --- | --- | --- |
| [LinkedIn: A declarative video editing API](https://www.linkedin.com/blog/engineering/media/declarative-video-editing-api) | Renderer-independent edit intent | Separate authoring, canonical IR, plan, and backend bundle |
| [OpenTimelineIO](https://opentimelineio.readthedocs.io/) | Timeline interchange and rational time | Typed timeline graph and exact time values; not used as the execution model |
| [MLT XML](https://www.mltframework.org/docs/mltxml/) | Declarative producer/playlist/tractor graph | First-class source/timeline ownership and backend-neutral composition |
| [FCPXML Resources](https://developer.apple.com/documentation/professional-video-applications/fcpxml-reference/resources) | Centralized typed media/effect resources | Project-owned resources with stable IDs and typed references |
| [FCPXML Story Elements](https://developer.apple.com/documentation/professional-video-applications/fcpxml-reference/story-elements) | Story ownership and connected media | Nested ownership plus explicit cross-owner relations |

## Agent-Oriented Authoring

| Source | Useful idea | VEAC consequence |
| --- | --- | --- |
| [Remotion](https://www.remotion.dev/) | Code-owned composition with deterministic rendering | Treat authoring as reviewable source and keep rendering deterministic |
| [Revideo](https://re.video/) | Programmatic video construction on a timeline | Closed animation/time primitives rather than imperative editor gestures |
| [JSON2Video](https://json2video.com/) | Machine-generated declarative video documents | JSON is useful for IR, but surface authoring needs stronger ownership and diagnostics |
| [OpenShot Cloud API](https://www.openshot.org/cloud-api/) | Video editing as structured remote jobs | Stable project IDs, artifact identity, and resumable execution matter |
| [Shotstack](https://shotstack.io/) | Timeline JSON with tracks, clips, and render outputs | Typed outputs and source/record separation are essential API boundaries |

## Transaction and Workflow Models

| Source | Useful idea | VEAC consequence |
| --- | --- | --- |
| [Blender Operator redo model](https://docs.blender.org/manual/en/latest/interface/operators.html) | Replayable operations with editable parameters | Typed edit batches with operation identity and preconditions |
| [DaVinci Resolve scripting](https://deric.github.io/DaVinciResolve-API-Docs/) | Projects, timelines, clips, render jobs, metadata | Keep editing, planning, and rendering as separate workflows |
| [Avid MediaCentral Cloud UX API](https://developer.avid.com/connector_api/CloudUX/index.html) | Versioned workflow integration | Explicit revisions and deterministic artifacts |
| [SMPTE ST 2067 / IMF](https://www.smpte.org/standards/st2067) | Composition playlists and package identity | Separate logical composition from physical media assets |

## Media and Backend Semantics

| Source | Useful idea | VEAC consequence |
| --- | --- | --- |
| [FFmpeg Filters](https://ffmpeg.org/ffmpeg-filters.html) | Rich execution substrate with strict stream/time semantics | Planner/codegen own backend details; authoring never emits filter strings |
| [FFprobe](https://ffmpeg.org/ffprobe.html) | Machine-readable stream metadata | Probe snapshots are explicit artifacts consumed by planning |
| [libavfilter design](https://ffmpeg.org/doxygen/trunk/group__lavfi.html) | Typed media pads and filter graphs | Validate visual/audio/caption component compatibility before codegen |
| [ICC color management](https://www.color.org/) | Explicit color spaces and transforms | Color pipelines declare input, working, output spaces and ordered stages |
| [EBU R 128](https://tech.ebu.ch/publications/r128) | Integrated loudness and true peak | Closed loudness processor parameters and unit types |

## Text and Captions

| Source | Useful idea | VEAC consequence |
| --- | --- | --- |
| [WebVTT](https://www.w3.org/TR/webvtt1/) | Timed cues, voices, and millisecond precision | Caption items use record spans; VTT retains speaker voice tags |
| [SubRip](https://en.wikipedia.org/wiki/SubRip) | Minimal timed text interchange | SRT is a deliberately constrained sidecar target |
| [ASS specification](https://github.com/libass/libass/wiki/ASS-File-Format-Guide) | Rich caption styling and positioning | Rich caption output has a faithful supported subset and fails closed otherwise |
| [Unicode text segmentation](https://unicode.org/reports/tr29/) | Grapheme versus scalar indexing | Animation can use grapheme units; canonical spans use explicit scalar indices |

## Templates and Interchange

| Source | Useful idea | VEAC consequence |
| --- | --- | --- |
| [Canva Apps Design Editing](https://www.canva.dev/docs/apps/design-editing/) | User-replaceable design elements | Template slots are owner-item constraints, not free-floating property bags |
| [Adobe After Effects Essential Graphics](https://helpx.adobe.com/after-effects/using/creating-motion-graphics-templates.html) | Explicit editable template controls | Media/text editability is opt-in and typed |
| [AAF](https://aafassociation.org/) | Stable object identity in editorial interchange | Typed IDs and canonical references are long-lived contracts |

## Rejected Directions

- Treating FFmpeg command text as the authoring language.
- Exposing arbitrary effect/property maps to preserve backend flexibility.
- Using JSON syntax directly as the agent-facing language.
- Maintaining a second legacy parser after the new authoring model is active.
- Copying an external NLE schema without preserving VEAC's canonical/planner/runtime boundaries.

Research links should be reviewed when they influence an accepted language primitive or planner contract. Product feature lists alone are not evidence that a mechanism is implemented.
