# 标准库与 Domain Opset

VEAC 的视频构造能力属于闭合、版本化标准库，不属于 lexer keyword，也不靠 JSON 字段反射生成。
Agent 应读取 `veac language-spec` 的 `standard_library` 与 `domain_opset`，再生成或修改 `.veac`；
不能从 runtime Rust 名称、canonical IR 字段或 FFmpeg 参数猜测调用方式。

## 两层合同

`standard_library` 描述源码可见名字：

- `types` 将 `Canvas` 等类型名绑定到 numeric domain type opcode；
- `free_functions` 发布 `canvas(...)`、`project(...)` 等自由函数；
- `methods` 同时发布 receiver 与 `with_sequence(...)`、`entry(...)` 等 method；
- 每个 callable 固定 operation opcode，并内嵌 ordered operands、result、instruction 与 effect；
- `domain_opset_version` 必须等于同一合同中的 opset version。

`domain_opset` 是 Core 与 runtime 共用的 ABI：

- `version` 当前为 `7`；
- `registry_digest` 是 lowercase SHA-256 identity；
- `types` 按 opcode 排序，固定 `opcode + name + container`；
- `operations` 按 opcode 排序，固定 canonical name 与完整 contract。

两个表由同一个 verified registry 生成。`LanguageSpec::validate` 会重新计算 digest，检查完整性、
唯一性、canonical 排序、558 个 free function 加 23 个 method 到 581 个 operation 的一一映射，
并与当前 build identity 比较。
未知字段、缺少操作、重复 opcode、receiver 漂移、operand 重排或 effect 漂移都会 fail closed。

## Typed Plugin Effect Inventory

`LanguageSpec.plugin_effects` 发布当前 build 唯一允许的 versioned plugin descriptor。它不是动态
插件加载入口，也不接受 parameter map 或兼容 alias。每个 descriptor 固定：

- descriptor constructor 与 application constructor；
- descriptor schema、schema version、namespace 与 implementation；
- content-addressed `effect_type` 与 lowercase SHA-256 `digest`；
- ordered typed parameters、有限 canonical bounds 与 `supports_curve` Temporal 能力；
- determinism 与有序 backend adapter 集合。

当前 closed inventory 只有 `plugin_reference_monochrome_v1()`，由
`video_plugin_scalar_effect(...)` 应用；参数 `amount` 是支持 Temporal curve 的 `number`，范围
`["0", "1"]`。bound 使用 canonical decimal string，避免 JSON float 表达漂移；backend 固定为
`ffmpeg-8`。descriptor 的全部执行语义进入 descriptor digest；Domain
registry v5 identity 进一步提交 canonical descriptor count、`effect_type` 和 digest。因此 runtime
registry 与 language-spec projection 不可能在插件集合上静默漂移。

## 当前 Domain 类型

完整的 214 项类型表只能从当前构建的 `veac language-spec` 读取。文档不复制 numeric opcode，
避免手写表与 verified registry 分叉。`Context` 是 host 创建的 graph-local value；`Project`、
`Sequence`、`Layer`、`Item` 是 graph container。`Resource`、`Relation`、`MulticamGroup`、`Apply`、
`Annotation` 与 `Delivery` 是具有单一 owner 的 graph entity，其余 Domain value 是不可变描述值。

`Context` 是 host 创建的 graph-local value。Domain handle 不可作为 public input、literal、map key
或普通表达式结果逃逸。`Project`、`Sequence`、`Layer`、`Item` 是 graph container；`Resource` 是
Project-owned graph entity，`Relation` 是 Sequence-owned graph entity；二者都不是 container。
`Source` 与 `TextStyle` 可 non-owning 引用 Resource，`Relation` 可 non-owning 引用 Item；
`ContentIdentity` 将源码声明的摘要与普通 text 隔离。源码不能投影任何 domain value 的存储字段。

## 调用与所有权

完整 581 项 callable inventory 只能读取 `veac language-spec`。method contract 的 `ordered_operands`
首项是
receiver，后续项严格对应显式参数。每个 operand 都发布 `shape` 与 `axis`：`topology` 可决定实体、
所有权、顺序或 graph shape；`leaf` 只能贡献内容参数。`domain_list` 是同一 DomainType 的有序列表，
不是任意 JSON array。

九个 singular owner method 都有对应的 plural method：Project 批量接入 Resource、Sequence、
MulticamGroup、Annotation 与 Delivery，Sequence 批量接入 Layer、Relation 与 Apply，Layer 批量接入
Item。plural call 在修改 owner 前验证完整列表，保留输入顺序，并在重复 key、重复 child、stale handle、
already-owned 或 cross-graph handle 出现时原子失败。

## 名字与词汇隔离

Domain type、function、method 作为标准库 symbol 的记录只出现在 `standard_library`。发布它们不会新增
`VocabularyEntry`，也不会改变 91 个 syntax spelling、97 个 syntax use 或空的
`lexer_keywords`。某个拼写同时被 closed descriptor position 使用时，`vocabulary` 只记录该
grammar use；它不会因为标准库同名 callable 而增加虚假的 keyword/use。

源码解析在 typed resolution 阶段将 standard-library name 解析成 numeric operation opcode；Typed
HIR 与 Core 此后不按字符串 dispatch。Core program、direct call 与 closure body 都固定相同 opset
version 和 registry digest，runtime 执行前再次核对 identity。当前 v7 digest 是
`58ba887ccc37ed3ce7d99104b83fc01267e455b1adf6f04e98303e1d33b074ee`。
