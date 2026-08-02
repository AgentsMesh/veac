# Programming Resource Budget

Compilation enforces these limits after every public `SourceLoader`; it never trusts a loader to
enforce them:

| Resource | Limit |
| --- | ---: |
| One source module | 16 MiB |
| Source graph | 1,024 modules and 64 MiB |
| Import depth | 64 levels |
| Constant, preset, or component-parameter dependency depth | 64 active levels |
| Resolved declarations in one dependency graph | 16,384 nodes |
| Surface or core tokens | 1,000,000 per input |
| Core authoring input | 32 MiB |
| Active component composition depth | 64 levels |
| Expanded component instances | 16,384 nodes |
| Component parameter evaluation | 65,536 evaluations per expansion |
| Definition validation | 16,384 preset or synthetic-environment units |
| Component definition expansion | 65,536 aggregate component instances |
| Definition validation work | 256 MiB of input and validated source |
| Resolved symbol storage | 64 MiB of logical retained bytes |
| Static replacements | 16,384 |
| Expanded source and owned rewrite working set | 32 MiB |
| Source-edit operations and text edits | 4,096 per batch |
| One source-edit expression | 64 KiB |
| Source-edit expression payload | 16 MiB per batch |
| Edited module output | 16 MiB |
| Source-edit string rewrite working set | 48 MiB |
| Source-edit JSON input | 64 MiB |
| Program or core diagnostics | 256 entries |

Dependency and component identity cycles fail before depth or node budgets. A deeply nested hygienic path may hit
the independent 128-byte canonical name limit first. A rewrite with no replacements reuses its owned
input. Before a replacement output is allocated, the old owned input, owned replacement values, and
target output are charged together; output is then written once in source order. Active component
frames share the same 32 MiB retained parameter-and-fill budget, so recursive parameter binding or
slot forwarding cannot reset the limit. Captured constant and preset environments are immutable and
shared; a parameter overlay contains only values materialized for that instance.
All component instances in one project expansion also share the parameter-evaluation budget. The
current instance completes its independently bounded binding checks before the aggregate charge, so
unknown, missing, type, expression, and dependency diagnostics retain precedence. A failed checked
charge is not committed, and component body or slot expansion does not begin after that failure.
Project assembly retains structured sequence bodies instead of wrapped copies, and charges the project,
sequence bodies, active component frame, and prospective final output together. With no generated
sequences, it returns the owned project buffer directly.
Definition-validation limits are shared by every imported and entry preset and component in one
compilation. One preset consumes one unit; a component consumes one unit per bounded synthetic
environment. Each unit is charged after semantic validation, so preset, component-cycle, and
definition diagnostics retain precedence. The unit crossing a limit remains bounded by the normal
parser and expansion limits; its charge is not committed and no later definition validation runs.
Resolved-symbol storage is shared across the complete source graph. A constant or expanded preset
charges its variable-size payload once when first materialized, after its semantic checks succeed.
Every first materialization and qualified alias also charges a fixed logical entry cost plus its
UTF-8 key bytes. Qualified import aliases therefore charge their new keys but share `Arc`-backed constant and preset
payloads, including cache hits and repeated aliases of one module. The logical budget is deterministic;
it is intentionally independent of allocator-specific map node sizes and string capacities.
The component catalog is owned once by the resolver rather than copied into cached scopes. Component
logical storage charges its declaration/interface payload and every captured name-to-key component
binding, including imported aliases, while shared constant and preset `Arc` payloads are not charged
again. Preset maps are keyed name-first, so wrong-kind lookup is bounded by the closed preset-kind
range instead of scanning all resolved presets.
Source-edit range, UTF-8 boundary, overlap, target, and expression checks run before aggregate byte
charges where their established diagnostic has precedence. The 48 MiB limit describes only the old
text, borrowed replacement payload, and prospective rewritten string used by the text rewrite. It is
not a bound on complete transaction heap usage, which also includes resolver state and compilation
artifacts. A successful preview retains the new compiled source graph, the previous edited module,
the prior module-name set, and revisions; it does not clone the complete previous source graph.
At the diagnostic limit, the final retained entry is a stable budget error and parsing stops when no
further diagnostic can be retained.
