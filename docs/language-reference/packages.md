# VEAC Package 合同与本地发现

VEAC package v1 是只读、可复现的本地 package 边界。package root 必须同时包含：

* `veac.package.json`：作者声明的 package identity、module 入口、精确依赖和 API digest；
* `veac.package.lock`：唯一依赖解析权威，固定 root 与每个依赖的完整文件清单、content digest、API digest、依赖图和编译器兼容轴；
* `veac.package.api.json`：由 compiler module interface 决定的 API metadata v2。

三个文件都使用 versioned、`deny_unknown_fields` JSON。重复 object key、未知字段、错误 schema identity 和错误 schema version 都会失败。解析接受等价的 JSON whitespace 与 key ordering，canonical encoder 只产生一种稳定表示。当前版本不增加第二套 VEAC import 语法，也不允许 package 安装或构建 hook。

## Manifest

manifest schema 是 `https://veac.dev/schemas/package-manifest/v1`，`schema_version` 为 `1`。

| 字段 | 合同 |
| --- | --- |
| `package` | `{ "name": "lower-case-name", "version": "x.y.z" }`；版本必须是完整 exact SemVer，不接受 `latest`、range 或隐式选择 |
| `entry` / `entry_sha256` | root-relative `.veac` module 入口及其 lowercase SHA-256；入口必须出现在 lock 的 `root_files` 中 |
| `dependencies` | 按 package identity 排序且不重复的 `{name, version}` 数组；同一个 name 不能出现两个版本 |
| `api_sha256` | `veac.package.api.json` canonical JSON 的 v2 package API digest |

package 入口必须使用 `module { ... }`。可执行 project source 不是可发布 module interface，因此 root 或 dependency 的 project entry 都会在 discovery 时失败。

## Lock

lock schema 是 `https://veac.dev/schemas/package-lock/v1`，`schema_version` 为 `1`。`root` 必须和 manifest 的 `package` 完全相同。顶层 `root_files` 和 `root_content_sha256` 固定 root package 的完整文件集合及其聚合 digest；root entry 必须包含在该集合内。root 文件不能位于依赖 package path 内，依赖 package path 也不能位于 root 文件路径之下。

`compatibility` 精确绑定当前 compiler contract：

* `language_version`：当前 `veac-lang` exact SemVer；
* `core_version`：verified Core 版本；
* `domain_opset_version`：闭合 Domain registry opset；
* `canonical_schema_version`：canonical project IR schema。

任一兼容轴漂移都会在读取 package source 前失败。`packages` 是按 exact identity 排序的闭合集合；每项包含 package-relative `entry`、排序且非空的 `files`、精确 `dependencies`、`content_sha256` 和 `api_sha256`。目录不能重叠或嵌套，合同文件本身不能列入 source 清单。

resolver 从 manifest 依赖开始验证 lock 中的闭合 DAG，拒绝缺项、孤立项、版本不一致、循环和 digest 漂移。所有锁定 `.veac` 在 discovery 阶段必须是 UTF-8；未锁定的 helper 不能被加载。resolver 不读取 environment、clock 或 network，也不执行安装、脚本、生成或动态 dependency resolution。

## 逻辑模块身份与物理定位

package source 的 compiler identity 固定为
`packages/<name>@<exact-version>/<package-relative-path>`。root package 与 dependency 使用同一种格式；
同一个 exact package 的同一 package-relative module，无论作为独立 root、位于哪个 vendor path，或同时被多少
个显式 root 复用，都必须得到同一个逻辑 ID。lock 中的 `path` 只是 physical locator，不进入
`LoadedSource.id`、import route、source revision、`ModuleInterface` 或 nominal receiver/type identity。

读取文件时仍使用当前 lock snapshot 的 physical locator。loader 每次读取都会重新执行 root confinement、
regular-file、inode/file identity、大小和 SHA-256 校验，再把 bytes 绑定到逻辑 ID。逻辑身份相同不表示可以
信任任意副本：多个 mount closure 出现同一 exact identity 时，entry、content digest、API digest 与直接依赖
必须形成完全相同的 portable contract；任何分歧在 mount 阶段 fail closed，读取失败也不会回退到另一个副本。

## API Metadata v2

API schema 是 `https://veac.dev/schemas/package-api/v2`，`schema_version` 为 `2`。`package_api_digest` 对 canonical JSON 加上 `veac.package.api.v2\0` domain separator 后计算 SHA-256。`exports` 按 closed kind 与 canonical identity 排序，包含：

* `function`：名称、显式参数、返回类型和 compiler 推导的 callable semantics；
* `method`：canonical nominal receiver、名称、显式参数、返回类型和 callable semantics；
* `type`：exported struct fields 或 enum variants；
* `constant`：名称与 compiler 验证的值类型。

只有当前 module 声明并 export 的函数、方法、type-name root 和常量会进入接口。imported function、type 或 method 即使可在实现中调用，也不会被隐式 re-export。method receiver 使用 `{ "module": source_id, "name": declared_name }` canonical nominal identity；receiver 不重复出现在显式参数数组中。参数、field 和 variant 顺序属于 ABI，默认参数以 `has_default` 表示。

类型模型允许 primitive、Domain、list、`range<int>`、`map<text|identifier, T>`、二元以上 tuple、named nominal type 和 function type。Domain type 同时携带 stable `opcode` 与 registry `name`，两者必须与当前闭合 opset 一致。function type 上的 `pure`、`local`、`emit`、`any` 是作者声明并经 type checker 验证的调用约束，不等同于下节对某个 exported implementation 的推导结果。

## Callable Semantics

每个 exported function 或 method 都保存 compiler 从 verified Core 推导的事实：

* `effect`：`pure`、`local_mutation` 或 `graph_emit`；
* `contains_local_mutation`：独立记录实现中是否出现 local mutation，即使最终 effect 被更强的 graph emission 覆盖；
* `result.shape` / `result.leaf`：分别是 `const`、`build` 或 `temporal` stage；
* `result.parameters`：与显式参数严格等长的 dependency records；
* `result.receiver`：function 必须为 `null`，method 必须存在。

每个 dependency record 有 `shape_from_shape`、`shape_from_leaf`、`leaf_from_shape`、`leaf_from_leaf` 四个位。它们描述返回值两个 axis 对输入两个 axis 的精确依赖，不能压缩成一个“依赖此参数”的布尔值。method 的 compiled slot `0` 是 receiver；显式参数 `i` 对应 compiled slot `i + 1`。

metadata 不从 source text 或 parser AST 猜测 semantics，也不伪造 description、example、localized text、slot 或 search metadata。常量 initializer 在验证与求值后不会作为发布 metadata 暴露，因此常量接口只包含其类型。

## Domain Capabilities

`domain_capabilities` 是 implementation execution requirements，不是 signature 中出现的 Domain types。compiler 从 exported functions 与当前 module 的 exported methods 出发，在 verified Core 上传递遍历：

* `DomainConstruct` 和 `GraphEmit` instructions；
* 被调用的 private 或 exported user functions；
* 默认参数 thunks；
* nested verified closure bodies。

遍历按 function ID 与 opcode 去重。输出按 opcode 排序，每项同时携带 opcode 和标准 Domain operation name；未知 opcode、name/opcode 不匹配、重复或超过闭合 operation 上限都会失败。未被 exported implementation 执行路径触达、仅出现在类型签名中的 Domain type 不会声明 capability。

## Compiler Trust Loop

`discover_package(root)` 在返回 `PackageDiscovery` 前执行完整 trust loop：

1. 严格验证 manifest、lock、依赖图、物理文件 identity、文件 digest、API JSON 与 API digest；
2. 从该不可变 discovery snapshot 构造一个 `PackageSourceLoader`；
3. 使用同一个 `CompilerDatabase` 把 root 与每个 dependency entry 编译成 `ModuleInterface`；
4. 将 immutable compiler interface 投影为 canonical API v2；
5. 要求投影结果与 authored `veac.package.api.json` 精确相等。

因此，即使作者同步修改 API JSON 与它在 manifest/lock 中的 digest，只要 export、receiver、type、default、effect、stage、dependency mask 或 capability 与源码不一致，discovery 仍会 fail closed。每个 dependency 都会独立编译和比较，即使 root 没有 import 它。空 module 的空接口是有效 package。

## Rust API

公共入口在 `veac_lang::package` 与 `veac_lang::program`：

* `parse_*_json` 与 `canonical_*_json`：严格解析、验证和 canonical encoding；
* `package_*_json_schema`：machine-readable manifest、lock 与 API schemas；
* `discover_package(root)`：完成全部内容验证和 compiler trust loop；
* `PackageSourceLoader::for_root(root)`：返回 lock-confined loader 与 root `LoadedSource`，每次读取都复核文件 identity 与 digest；
* `CompilerDatabase::module_interface(root, loader)`：返回 immutable、compiler-owned exported module interface；
* `package::api::from_module_interface(package, interface)`：把 compiler interface 投影为 canonical API v2 model。

loader 的普通相对 import 只能解析到 importer owner package 内的锁定文件。跨 package import 必须使用
`package:name@version/path.veac`，并通过 importer owner 的直接依赖边验证，不能借物理 vendor path 读取
transitive 或 peer package。host 也可通过 exact `PackageIdentity` 选择显式挂载 root 的 entry。

## Project 与 package 组合加载

宿主需要同时加载 executable project 和已验证 module package 时，使用
`veac_lang::program::CompositeSourceLoader`。先用 `FileSystemLoader` 创建 project loader，再将
`PackageSourceLoader::for_root` 返回的 loader 以 exact `PackageIdentity` 挂载。项目 source 可以显式
请求 `package:name@version/path.veac`；该请求不改变 `.veac` parser，且返回的 canonical source ID
为 `packages/name@version/path.veac`。package 内部的相对与 qualified import 都由 importer 所属的 package
lock closure 处理；相对 import 不跨 owner，qualified import 继续检查 direct dependency edge、文件 identity
和 digest。

package 命名空间是 source graph 的一部分，不允许 project 占用。两个 root 可以共享同一 canonical package
source ID，但它们对该 exact package 的 portable contract 必须完全一致；compiler graph 因而只产生一个
canonical module node。版本必须完整匹配 lock 中的 exact identity；未挂载、路径越界、未声明依赖、
closure 分歧或 digest 漂移都会 fail closed。

CLI 的 source commands 与六个 `project` 子命令都不执行隐式 package discovery。每个 root 必须通过
可重复的 `--package-root` 显式给出；exact identity 来自已验证 manifest/lock，同一 identity 挂载
两次会失败。不同 identity 按 canonical identity 排序，因此参数顺序不能改变 complete compilation
identity、source index 或 project action。package source 每次读取都会重新验证，并进入
`complete_source_graph_sha256`，但不会进入 `authored_source_graph_sha256` 或可编辑模块清单。

Project workspace 额外把 portable package revision 写入每个 action：root 与 locked closure 都记录
exact identity、package-relative entry、entry/content/API digest 及排序的直接依赖。宿主 root
绝对路径不序列化，只由运行期 `ProjectPackageSet` 持有。manifest 编译、target planning、cache
identity 与 backend execution 使用同一集合，并在各边界重新 discovery；内容、API、闭包或文件
identity 在规划后漂移都会 fail closed。package root 也不能与工程的 source、material、build、
cache 或 delivery authority 相同、互为祖先。
