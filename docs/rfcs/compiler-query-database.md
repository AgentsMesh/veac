# 编译查询数据库

状态：source revision、lossless syntax、module interface、typed HIR 与 verified Core 的有界查询缓存已接线；source-edit 已支持显式复用数据库。

## 目标

VEAC 的长驻 Agent、IDE 和 source-edit host 需要重复编译同一 source graph。`CompilerDatabase`
提供显式、进程内、线程安全的有界查询缓存；普通 `prepare_*` API 仍创建一次性数据库，因此 CLI
单次调用的语义和资源生命周期不变。

缓存是性能层，不是语义层。关闭缓存、容量淘汰、进程重启或不同查询顺序都必须得到相同的诊断、
source revision、Core、provenance 与 canonical JSON。clean build 始终是等价性 oracle。

## 查询层级

允许缓存的值必须与 source bytes 和稳定编译器 ABI 完全决定，且不能携带一次执行的身份：

```text
允许：lossless source/CST -> module interface -> typed HIR -> verified Core
禁止：BuiltProgram、GraphHandle、DomainGraphTransaction、FrozenDomainGraph、运行预算状态
```

缓存查询按依赖关系分层。`SurfaceFile` 是 lossless CST；`ModuleInterface` 由完整 source graph revision、
canonical import resolution edges 和 compiler identity 决定；typed HIR 与 verified Core 继续使用内容寻址输入。
当前 interface 查询的 key 仍覆盖完整 graph，HIR/Core 的 key 仍覆盖一次 function batch 及其完整 context，
不是按 module 或 declaration 拆分的细粒度 query。module resolution、
source graph budget、visibility、类型/effect/stage 检查、Core verification、build-input binding、graph
execution、freeze、Temporal residualization、canonical lowering 与 `veac_ir::validate` 每次仍完整执行。

数据库同时维护 source ID 到精确 revision 的映射，以及完整
`(importer, requested_path, resolved_source_id)` 路由边。resolved source ID 投影形成 direct dependency
双向图；requested path 与 resolved source ID 任一变化都会失效 importer 及其 transitive importer。依赖
源码 revision 改变时，也只把 transitive importer 标记为 semantic invalidation；importer 自身字节未变，
因此它的 lossless syntax query 仍可命中。Interface key 包含完整 source graph revision 和 resolution route；
HIR/Core key 包含全部可见语义输入与 authored diagnostic provenance。受影响层 miss 后才消费对应
invalidation；这只描述当前缓存的失效边界，不代表独立模块或声明可以单独复用。累计 invalidation telemetry 不因 `clear()` 丢失，pending 集合则必须
清空。任何 query 都不能在 stale semantic value 上继续命中。

## Query Key

syntax key 包含查询 domain、crate version、language schema version、canonical source ID 和精确 UTF-8
source bytes。Interface key 另外包含完整 `(importer, requested_path, resolved_source_id)` 边集。SHA-256
只作为稳定摘要；key equality 同时比较原始 source bytes，不能把摘要碰撞当作 source 相等。同字节但
不同 source ID 或不同解析路由的模块不能共享，因为 source ID 参与诊断、module identity 和 provenance。

解析失败不缓存。类型或执行失败可以命中 syntax cache，但仍重新产生确定性诊断。source graph
预算、source ID collision、import cycle 与 loader confinement 在每次 prepare 时重新检查，cache hit
不能绕过安全验证。

## 资源合同

默认预算为 syntax 4,096 entries/64 MiB、Interface 1,024 entries/16 MiB、HIR 4,096 entries/64 MiB、
Core 4,096 entries/64 MiB，全部使用确定性 FIFO 淘汰。syntax bytes 计入完整 retained graph，而非仅
source bytes。依赖图另有 65,536 retained entries 和 32 MiB 上限；source revision、direct/reverse edge、
route 的 requested/resolved 字符串、pending invalidation 以及保守容器开销全部计费。单个 query 超过
容量时绕过缓存但继续正常编译；缓存压力不能把合法程序变成编译失败。统计发布各查询层 hit、miss、
insertion、eviction、bypass、当前 entry/bytes，以及依赖图当前 entry/bytes、reset、eviction 与 bypass。

依赖图压力采用确定性整图 reset，不做会留下半条 reverse edge 的局部淘汰。reset 后先尝试保留当前
观测；当前观测自身超预算则 bypass。发生 reset 后 Interface/HIR/Core 一并释放，并进入保守模式：后续
每次 source/route observation 都先清空这些语义缓存，查询因而重算，直到显式 `clear()` 开启新的依赖
观测周期。因而失去的只是增量性能，缓存容量、查询顺序和图 reset 永远不能成为正确编译的前提。

所有查询缓存受同一个 lifecycle `RwLock` 和不可回绕的 epoch marker 保护，固定锁顺序为
`lifecycle -> syntax/interface/HIR/Core/dependencies`。query 开始捕获 epoch，lookup 和 insert 持 read guard
并校验 marker；`clear()` 与依赖图 reset 持 write guard、推进 epoch 后清理缓存，因此 clear/reset 返回后，
此前已开始的 query 不能 late insert。dependency mutation 也在 write guard 内完成，reset 决策与语义缓存
清理之间不存在 stale hit 窗口。

依赖路由并发合同使用两个彼此独立、不可回绕的 identity。`ObservationToken` 标识某 source revision
的这一次观测，即使 source bytes 相同，每次观测也产生新 token；route writer 只有在 lifecycle epoch、
精确 revision 和 observation token 都仍为 current 时才能提交，因此 `A -> B -> A` 不能重新接纳第一次
观测 A 的 late writer。`InvalidationGeneration` 只在 route 或依赖语义变化时创建，并随 pending work
保留；它不因相同 revision 的重复观测而推进。消费 pending work 必须同时匹配 current observation token
和该 pending generation，旧 consumer 既不能消费较新的失效，也不能清除另一轮观测正在处理的工作。

`statistics()` 持有 lifecycle write guard，再按固定顺序读取全部缓存和依赖图。因此返回值是
syntax/interface/HIR/Core/dependency 的单一原子边界快照；统计期间新 query 会等待，不能形成跨层撕裂的
混合快照。这个操作适合 telemetry 与测试，不应放在编译热点的逐节点循环中。

`clear()` 替换 FIFO/map 容器以真实释放 backing allocation，释放 retained query 和 dependency graph，并结束
conservative epoch，但保留累计 telemetry。任何 retained accounting 的 checked arithmetic 溢出一律视为
不可缓存；`usize::MAX` 预算不表示可以接纳已经溢出的计费值。并发相同 query 可以重复进行无副作用的
parse，最终只保留一个 `Arc<SurfaceFile>`；调用者永远看不到未完成或部分初始化的值。

## 长驻 Host 生命周期

长驻 Agent、IDE 或 source-edit 服务应在进程级创建一个 `CompilerDatabase`，并把它显式传入每次
`prepare_*`、`module_interface` 和 `prepare_source_edit_path`。数据库只拥有可重建的编译查询；host
不得把 `ExecutableBuild`、`BuiltProgram`、graph handle、transaction、freeze 结果或 runtime budget
写入数据库。候选编辑执行前后都必须重新绑定输入、创建新的 graph transaction、freeze、lower 和验证，
因此同一 candidate 的 preview 与 consuming execute 结果相同但执行状态不共享。

一次性 CLI 可以继续使用默认临时数据库。缓存命中只减少重复编译，不改变 source-of-truth、diagnostic、
provenance 或 canonical IR。

## 当前边界与下一阶段

当前实现提供的是有界、可丢弃的性能缓存和正确的失效/并发屏障：相同完整 source graph 重复准备时，
lossless syntax、完整 graph-bound module interface，以及一次完整 function batch 的 HIR/Core 可能命中。
依赖 revision 变化会按 route 图标记 importer 的语义缓存失效，但不会把 HIR/Core 自动切成未变化声明的
增量结果。每次 build 仍完整完成 source graph resolution、graph verification、execution、freeze、
Temporal residualization、canonical lowering 和 IR validation。

clean build 是永久语义 oracle。缓存开启与关闭的结果必须做 differential test；不得缓存 graph-affine
value，也不得让缓存成为唯一可用的编译路径。真正按 module/declaration 复用 interface、HIR/Core 的
细粒度 query DAG、增量依赖传播和增量 lowering 属于下一阶段设计，不应由当前 API 或 telemetry 推断为
已实现能力。
