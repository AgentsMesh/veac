# VEAC 语言参考

VEAC 是面向 Agent 的确定性视频编辑语言，不是文本化 JSON。source graph 保留可复用声明、
编辑意图、source span、省略状态和闭合 variant；canonical JSON 是执行 IR。

当前生产路径只有 executable frontend：

```text
Surface -> typed HIR -> verified Core v10 -> bounded graph build
        -> freeze + authored temporal residualization -> canonical JSON IR
        -> canonical validation -> plan -> FFmpeg/artifacts
```

当前 canonical project envelope 使用 schema version 10 和 minimum reader 10。centered transition 只接受
centered true-overlap：两条相邻真实视频流必须完整覆盖交集窗口，backend 不用 held-frame `tpad` 补端点。

核心代数为：

```text
Project   = settings + resources + entry + multicams + sequences + annotations + deliveries
Delivery  = sequence + optional raster + typed artifacts
Sequence  = layers + relations + applies
Layer     = ordered items + optional audio routing
Item      = source + record span + optional source mapping + modifiers + optional template slot
Parameter = constant<T> | curve<T>
```

reference 由 grammar position 定型，例如 `resource`、`sequence`、`layer`、`item`、`group`、
`angle`、`track`、`bus`。未知 kind/field、重复字段、错误单位和未解析引用都会失败。

- [Project 与 resources](project.md)
- [工程工作区、target DAG、素材派生与 CAS](project-workspaces.md)
- [EvidenceSuite、证据 bundle 与验收 gate](evidence.md)
- 版本化语法机器合同：[Versioned language vocabulary](vocabulary.md)
- [标准库名字、Domain 类型与 numeric opset](standard-library.md)
- [可执行 Build、Core v10、Effect/Stage 与 graph transaction](executable-build.md)
- [声明式 Build input、Context 边界与 host manifest](build-inputs.md)
- [多语言项目、闭合 locale catalog 与构建矩阵](localization.md)
- [可执行 Temporal residualization、typed binding 与 cache identity](executable-temporal.md)
- [Temporal 动态叶 target 与属性矩阵](executable-temporal-sinks.md)
- [Modules、typed functions、values 与 executable 编程模型](programming.md)
- [Package 合同、锁定依赖与本地发现](packages.md)
- [有界集合操作与 lexical iteration](programming-collections.md)
- [Nominal value、exhaustive match 与静态 method](programming-nominal.md)
- [Typed component、模块工厂与复用](programming-components.md)
- [Source-of-truth editing](source-editing.md)
- [稳定 source addressing](source-addressing.md)
- [Timeline 与 mapping](timeline.md)
- [Sources](sources.md)
- [Modifiers 与 relations](modifiers-relations.md)
- [Text、captions 与 audio](text-caption-audio.md)
- [Deliveries 与 artifacts](outputs.md)

当前 executable opset v8 是 214 个 DomainType/582 个 operation 的闭合 registry；标准库 symbol
不计入 keyword，机器客户端通过 `veac language-spec` 读取合同。typed `animate` declaration 已发布，
Surface 不得把表达式字符串或 property bag 写入 canonical IR。每个公开机制都应在
[`examples/`](../../examples/) 中有可执行 source。`make check-examples` 会解析 source graph、
机器客户端按唯一 executable frontend 执行 verified build，并校验 canonical project。
