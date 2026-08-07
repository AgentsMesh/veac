# 稳定 Source Addressing

`source-index` 与 `SourceEditBatch` 只使用闭合、module-qualified semantic path。byte range 是一次
inventory 的观察值，不是公共 identity；格式化、注释和无关声明移动后，Agent 应读取新 revision 与 range，
但 typed target 不变。

## Name 与 Module

target 内每个 name 都遵循 declaration-name 合同：1 到 128 个 ASCII bytes，匹配
`[A-Za-z_][A-Za-z0-9_]*(-[A-Za-z0-9_]+)*`，并排除 reserved `true`/`false`。leading underscore 与
kebab-case 合法；dot、Unicode、leading digit、trailing dash 和 repeated dash 不合法。

module 是 loader 生成的 source-root-relative canonical ID，不放进 declaration name。它不能是 absolute、
`.`、`..`、空 segment、backslash、control character 或 `.veac-source.lock`，也不能逃出 source root。

## Declaration Path

```text
input:              input
constant:           constant
function:           function
method:             receiver + method
implementation:     receiver + implementation
struct:             structure
struct_field:       structure + field
enum:               enumeration
enum_variant:       enumeration + variant
enum_variant_field: enumeration + variant + field
```

module 与上述 path 必须选择恰好一个 authored declaration。method body 独立寻址；完整 impl block 使用
源码中必填的 `@identity` 形成 implementation target。同一 receiver 可拥有多个不同 identity 的 impl
block；重复 `(receiver, identity)` 会在编译与索引阶段 fail closed，而不会按 occurrence number 猜测。

input、constant、function、implementation、struct 与 enum 也是 v8 module inventory 中的顶层
declaration target，可用于 `set_top_level_declaration`、`remove_declaration` 和 declaration anchor。
nominal member target 继续用于更小粒度的 `set_declaration`。

## Import Path

import 的 authored identity 为：

```text
import: module + local alias
```

path 是 precondition 中需要精确比较的 source value，不是 identity。这样 Agent 可以用
`import_equals` 确认 `{ module, alias }` 仍指向预期 path，再在一个 batch 中 remove old alias、insert new
alias，并更新 use site。被导入 declaration 的 identity 始终使用其真实 source module，不使用调用方 alias。

## Temporal Path

animate body 使用完整 temporal declaration target：

```text
clip/text:       project + sequence + layer + item
clip mask:       item path + mask ordinal
clip effect:     item path + effect key + parameter key
apply:           project + sequence + apply key
apply mask:      apply path + mask ordinal
apply effect:    apply path + stage key + effect key + parameter key
site:            temporal_animation + closed property
```

例如同一个 Item 的 `visual_opacity` 与 `visual_position` 是两个不同 `BodySite`。完整 target 防止不同 mask、
effect parameter、apply、sequence 或 layer 的 leaf 冲突；ordinal 与 key 属于静态 topology identity。

完整 animate statement 另有顶层 declaration identity：

```text
temporal: project + sequence + layer + item + closed property
site: temporal_declaration
```

同一个 temporal target 用于 `set_body` 只替换 `{ ... }`，也用于 `set_declaration`、
`set_top_level_declaration` 或 remove/anchor 完整 animate。canonical ProjectId、SequenceId、ItemId 与
binding/program ID 都从 logical owner 派生，但 source edit 不使用这些执行后 ID。

## Typed Anchor

插入操作不能传 byte offset，只能选择：

```text
module_start | module_end
before_declaration(target) | after_declaration(target)
before_import(target)      | after_import(target)
```

anchor target 的 module 必须等于 operation module。module range 对 entry 覆盖整个文件；对 imported
`module { ... }` 只覆盖 brace 内 body，因此 `module_start`/`module_end` 不会破坏 wrapper。

## Runtime Topology

普通 function body 中构造的 Item、Layer、Resource、Effect 和 Delivery 是 runtime graph value，不会被
反射成独立 source declaration。要修改它们，应编辑 owner function/method body、被引用 const 或 typed
animate declaration。VEAC 不从 canonical JSON field 猜 source field，也不把 derived ID 或 raw range
写回 source-edit target。
