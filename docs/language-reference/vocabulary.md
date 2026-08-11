# VEAC 语言词汇合同

VEAC 将固定语法词汇发布为可版本化、可校验的机器合同。Agent 不需要扫描 parser、formatter
或 examples 来猜测语法；应从正在运行的同一版 VEAC 读取合同：

```bash
veac language-spec
veac language-spec --schema
veac schema --contract language-spec
```

`veac language-spec` 输出 RFC 8785 canonical JSON，并且恰好带一个尾换行。同一 build 的重复
输出逐字节一致。两个 schema 入口输出相同的 strict JSON Schema。当前合同标识为
`https://veac.dev/schemas/language-spec`，`schema_version` 为 `7`，不提供旧 schema 兼容层。

## V7 数据模型

顶层 `LanguageSpec` 包含 `schema`、`schema_version`、`language`、`language_version` 和
`vocabulary`、`standard_library`、`domain_opset`、`plugin_effects`。`SyntaxVocabulary` 包含 `lexer_keywords`
与按 spelling 排序的 `entries`。
同一个 spelling 永远只有一个 `VocabularyEntry`，不同 owner 下的语义由多个 `SyntaxUse`
表达，不能复制 entry，也不能把 owner 信息编码进字符串。

| 字段 | 含义 |
| --- | --- |
| `spelling` | 源码中的精确拼写 |
| `identifier_policy` | 该拼写在词法上能否作为用户 identifier |
| `uses` | 按稳定顺序排列且去重的全部精确语法用途 |
| `category` | 词汇类别 |
| `layer` | 由 grammar position 唯一决定的语言层 |
| `position` | parser 所拥有的精确语法位置 |
| `canonical_role` | spelling 在该位置承担的角色 |

`LanguageSpec::validate` 只接受当前 build 生成的完整合同。缺少 entry/use、篡改 layer、position
或 role、加入 lexer keyword、漂移 standard-library/opset identity、使用其他 language version，
或提交未知 JSON 字段都会失败。domain 合同见 [标准库与 Domain Opset](standard-library.md)。

## 词法与标识符

`lexer_keywords` 必须等于 `[]`。VEAC 不通过全局 keyword 表给普通 word 定义含义，而是在
typed grammar position 中解释它。全局 reserved literals 只有 `true` 和 `false`；其他固定
语法 spelling 在其 owner 之外仍可满足普通 name 规则。

| `identifier_policy` | 当前 spelling 数 | 规则 |
| --- | ---: | --- |
| `allowed` | 88 | 具有合法 name shape，可在其他 owner 中作为用户 identifier |
| `reserved` | 2 | 全局禁止作为 identifier；当前仅 `true`、`false` |
| `not_applicable` | 1 | 不具有 identifier shape；当前为 `%` |

这一模型避免了“新增语法词就破坏用户命名”的全局保留字扩张。identifier/reference 仍须通过
name shape、owner、类型与解析后的引用校验。

## 固定语法原语

当前 build 的 contextual control 由 `ControlUse` 单一发布，精确绑定
`spelling + position + canonical_role`。executable parser、source index 和诊断
必须消费 typed `ControlUse`，不得直接比较、拼接或输出注册 spelling。
`fn` 在 `static_declaration` position 中是 `declaration_introducer`，并与 `list`、`map`、`range` 在
`structural_type_constructor` position 中作为 `kind_discriminator`。函数签名使用的 `:` 和 `->`
是 punctuation，不是 lexer keyword 或 vocabulary entry。`value_type` category 仍只包含十个
primitive spelling；递归 semantic type 由这些 primitive 与 constructor 组合，不复制 token entry。
表达式位置的 `fn` 还是 `expression_closure_introducer` 的 `clause_introducer`。
函数 effect 的 `effect` 是 `expression_function_effect_clause` 的 `clause_introducer`；`pure`、
`local`、`emit`、`any` 是 `expression_function_effect_value` 的闭合值，不是全局 keyword。
范围步长的 `by` 是 `expression_range_step` position 的 `clause_introducer`；`..` 是 punctuation，
不进入固定词汇表。

Surface grammar 中真正的有限闭集由共享 `SyntaxToken` 实现；当前 `enum_value` 是函数 effect 的
`pure`、`local`、`emit`、`any`。field introducer、declaration introducer 与 reference kind 不是
枚举；即使它们只有少数当前拼写，也必须是 owner-scoped `ControlUse`。动态 identifier、reference
ID、文本、数值和表达式不是固定词汇 entry。

`standard_library` 的 214 个 type symbol、559 个 free-function symbol 和 23 个 method symbol 是
typed name-resolution 合同，不是 keyword，也不会仅因进入标准库而进入 `vocabulary`。例如
`Canvas`、`video_resource`、`with_sequence` 由类型检查器在对应的 type/callee/method 位置解析；
它们不会出现在 `lexer_keywords`。582 个 numeric domain operation 也是 backend opcode 合同，不是
源码 token。若某个标准库 symbol 恰好与固定语法同名，`VocabularyEntry` 仍只记录它真实的语法 use。

内部仍可保留旧 Core authoring descriptor 供实现代码迁移，但它们不是当前 `.veac` Surface
production，不得成为 `LanguageSpec` entry/use，也不得出现在 public JSON Schema。生产 lowering
只消费 executable program 与 verified Core，不能把 descriptor 拼成旧 property source 后重新解析。

## 当前统计

当前 `language_version` `0.1.0` 发布 `92` 个不同 spelling 和 `98` 个精确 syntax use。
category count 是“至少具有该 category 一个 use 的不同 spelling 数”；同一 spelling 可出现在
多行。第三列是该 category 的精确 use 数。

| Category | Spelling count | Exact uses |
| --- | ---: | ---: |
| `reserved_literal` | 2 | 2 |
| `contextual_control` | 59 | 62 |
| `enum_value` | 4 | 4 |
| `builtin_function` | 14 | 14 |
| `unit_suffix` | 6 | 6 |
| `value_type` | 10 | 10 |

公开语言层只有 `static_program`、`executable_expression`；两个层分别有 60、38 个 exact use。
canonical roles 为
`boolean_literal`、`clause_introducer`、`closed_value`、`compile_time_type`、
`declaration_introducer`、`declaration_modifier`、`receiver_binding`、
`kind_discriminator`、`numeric_unit`、`pure_function`、`reference_kind`。

## 语法位置

当前合同有 29 个 inhabited positions：`static_program` 13 个、`executable_expression` 16 个。
以下清单按 layer、spelling 排序，且必须与运行时 public inventory 逐项一致。

`executable_expression`：`expression_boolean_literal`, `expression_closure_introducer`,
`expression_else_branch`, `expression_for_binding`, `expression_for_source_clause`,
`expression_function_callee`, `expression_function_effect_clause`, `expression_function_effect_value`,
`expression_if_branch`, `expression_let_binding`,
`expression_match_introducer`, `expression_mutable_assignment`, `expression_mutable_binding`,
`expression_numeric_unit`, `expression_range_step`, `expression_temporal_attachment`。

`static_program`：`build_input_role_position`, `import_alias_clause`, `module_declaration`,
`method_receiver`, `module_member_modifier`, `static_declaration`, `structural_type_constructor`, `temporal_property_position`,
`temporal_source_clause`, `temporal_source_kind`, `temporal_target_clause`, `temporal_target_kind`,
`value_type_position`。

## Agent 消费规则

Agent 必须加载目标 build 的 `language_version` 合同，按 `layer`、`position` 和
`canonical_role` 选择 spelling，并保留 `identifier_policy`。不能从 IR 字段名、FFmpeg 参数、
示例或产品术语反推固定语法。合同完整描述固定词汇；字段必选性、声明顺序、引用可达性、单位与
跨字段约束由 [executable grammar](../language-design/grammar.md)、
[programming grammar](../language-design/programming-grammar.md) 和语义校验共同定义。
