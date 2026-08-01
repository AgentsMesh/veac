# Capability Execution Notes

This document records cross-cutting behavior that is too detailed for the capability index. The
evidence files remain authoritative for implementation and test links.

## Resolution And Media Authority

The CLI resolves exactly one named render config. Reachability is folded independently through
nested sequences before media I/O, so disabled, muted, solo-suppressed, audio-ineligible, and
unselected-output materials do not trigger probing. Reachable local media is SHA-256 pinned and
probed without mutating canonical JSON. Authored paths are reverified after observation.

One invocation pins one ffprobe master. Safe protocol and demuxer allowlists reject indirect child
resource inputs such as HLS, ffconcat, and DASH. Remote locators require a verified local binding
from an external materialization provider; resolution itself performs no network acquisition.

## Planning And Backend

The resolved plan carries global stream indexes, SAR/rotation, source-time curves, synthesis policy,
nested canvas/rate, composition, effects, transition handles, multicam switches, audio routing,
color pipelines, and exact duration. Codegen accepts serialized plans as untrusted and preflights
references, arithmetic, resource use, graph complexity, source mappings, and output compatibility.

Backend commands are argv data, never shell strings. `emit_all` seals every deliverable together;
runtime alone converts validated tasks into executable invocations. Portable `auto` currently binds
to software. An explicit hardware requirement stays fail-closed until device and upload-path facts
are represented in the backend plan.

## Budgets

Canonical validation and codegen both cap sequence duration, output frames, pixel-frames, audio
samples, reverse intervals, visual intermediates, tracks, clips, effects, masks, keyframes,
source-curve segments, and caption cues. Overflow and mutated serialized plans fail before filter
construction. Text shaping/ASS emission, graph size, arguments, artifacts, provider I/O, and runtime
deadlines have separate bounded contracts.

## Media Semantics

Freeze and hold select the containing displayed frame at canvas rate before cloning it. The project
timebase is source-time precision, not output fps. Cubic easing is split with de Casteljau
restriction and evaluated from the same canonical curve in edit, plan, and FFmpeg expression paths.

Effect parameter modes are registry capabilities. The generated canonical schema exposes exactly
number, number-curve, boolean, and color values. Every built-in video number accepts a curve;
static-only parameters reject curves. Registry keys and parameter modes must exactly match the V2
authoring surface and executable backend catalog. Range-limited command-driven filters use
instance-addressed runtime commands. Composition uses high-depth color/alpha paths and preserves
straight alpha through blend, masks, mattes, scoped Apply pipelines, and output conversion.

## Delivery And Commit

Resource authority is computed per deliverable from its visual, audio, caption, font, and LUT
closure. Runtime snapshots only that SHA-256-bound set, rebinds typed paths to private copies, pins
tool masters, and rechecks identities around action, checkpoint, and commit boundaries. Two-pass
tasks require exact pass numbers and one passlog family.

Each task stages and commits atomically. Deterministic output locks, durable recovery journals,
descriptor-relative no-follow mutation, directory identity checks, deadlines, rollback, and
case/Unicode alias detection protect concurrent output. Earlier successful deliverables are not
rolled back if a later task fails; resumability is task-scoped, not a bundle-wide transaction.

Artifact and proxy substitution is content addressed and role specific. Full render-segment reuse
requires exact plan, profile, range, fidelity, provenance, digest, and size agreement. Corruption,
post-render replacement, and absent required artifacts fail closed.

## Provider Boundary

Providers are verified local processes using a bounded manifest/execute protocol. Negotiation,
executable identity, canonical I/O, total deadlines, isolated staging, artifact commit, source and
evidence validation, and atomic proposal trial-application are VEAC responsibilities. Model engines,
weights, inference quality, and remote media acquisition remain provider responsibilities.
