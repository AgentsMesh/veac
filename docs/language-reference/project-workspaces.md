# 工程工作区

工程工作区把多个可执行 `.veac`、素材派生任务、验收合同和交付物组织成一个确定性 DAG。
它不替代时间线入口 `main(Context) -> Project`：工作区入口负责“构建什么”，时间线入口负责“视频是什么”。

```text
project.veac: workspace() -> ProjectManifest
  -> profile / locale / matrix 实例展开
  -> typed target DAG
  -> VeacRender | MediaDerivation | Evidence
  -> verified CAS
  -> file / directory delivery + canonical receipt
```

## 目录与入口

建议让手写源码、素材、过程数据和交付物拥有不同根目录：

```text
project/
  project.veac
  src/                 # main.veac、evidence.veac 及其模块
  materials/           # 正式输入素材
  build/               # 可删除的 staging
  .cache/veac/         # verified CAS、计算记录和 lease
  dist/                # 面向人的交付物
```

`project.veac` 必须提供根模块本地的 `workspace() -> ProjectManifest`。`ProjectManifest` 等宿主 ABI
类型由 `veac/project.veac` prelude 提供；工程文件仍可导入普通模块、调用纯函数和复用 nominal value。

```veac
fn workspace() -> ProjectManifest {
  ProjectManifest {
    schema: "veac.project", version: 1, id: identifier("campaign"),
    paths: ProjectPaths {
      source_base: "src", material_root: "materials", build_root: "build",
      cache_root: ".cache/veac", delivery_root: "dist",
    },
    defaults: ProjectDefaults {
      profile: OptionalIdentifier.Some { value: identifier("preview"), },
      locale: OptionalIdentifier.Some { value: identifier("zh-cn"), },
      max_instances_per_target: 16, max_total_instances: 64,
    },
    locales: [
      ProjectLocale { id: identifier("zh-cn"), language_tag: "zh-CN", },
      ProjectLocale { id: identifier("en-us"), language_tag: "en-US", }
    ],
    profiles: [ProjectProfile {
      id: identifier("preview"),
      execution: ExecutionPolicy.Parallel { max_tasks: 2, },
      proxy: ProxyPolicy.PreferExisting,
      segmentation: SegmentationPolicy.Whole,
    }],
    targets: [ProjectTarget {
      id: identifier("render"),
      entry: ProjectTargetEntry.Veac { source: "main.veac", },
      localized: true,
      inputs: [
        ProjectInput { id: identifier("profile"), source: ProjectInputSource.ProfileBinding, },
        ProjectInput { id: identifier("locale"), source: ProjectInputSource.LocaleBinding, },
        ProjectInput { id: identifier("theme"), source: ProjectInputSource.MatrixBinding { axis: identifier("theme"), }, },
        ProjectInput { id: identifier("hero"), source: ProjectInputSource.ProjectMaterial { path: "hero.mov", }, }
      ],
      profiles: [identifier("preview")],
      axes: [MatrixAxis { id: identifier("theme"), values: [identifier("light"), identifier("dark")], }],
      needs: [],
      outputs: [ProjectOutput.Media { id: identifier("video"), media_type: MediaType.Video, }],
      deliveries: [ProjectDelivery.File {
        id: identifier("preview-file"), output: identifier("video"),
        destination: "{target}/{profile}/{locale}/{axis.theme}.mp4",
      }],
    }],
  }
}
```

五个路径都相对工程文件所在目录解析，必须留在工程根内。`source_base` 和 `material_root` 必须已存在；
`build_root`、`cache_root`、`delivery_root` 可由 CLI 创建。五个 authority 的 canonical 根目录不能相同、
互为祖先或通过符号链接重合，因此 `.` 不能作为其中一个根。过程文件不能混入源码树，CAS payload
也不会被当作正式素材。

## 实例矩阵与 DAG

一个 target 按 profile、locale 和所有 matrix axis 的笛卡尔积展开。`localized: true` 使用 manifest
中的全部 locale；`profiles` 为空时使用默认 profile；实例数受两个显式 budget 限制。交付路径模板在
实例化后展开，任何冲突都会失败，不能让两个实例覆盖同一目的地。

`needs` 只表达执行顺序；`ProjectInputSource.Artifact` 和 `AnalysisFact` 同时建立依赖边并绑定上游
output。`TargetRef` 通过 `Same`、`Exact` 或 `AllMatching` 选择实例；非唯一选择必须显式声明。
所有引用先解析成无环、稳定排序的 graph，随后才允许执行。

输入 authority 是闭合的：`Literal`、`ProfileBinding`、`LocaleBinding`、`MatrixBinding`、
`ProjectMaterial`、`Artifact`、`AssetFact`、`AnalysisFact`。普通值必须精确匹配目标源码的 `input`
声明；素材必须匹配 material role，且不能通过 CLI inline value 伪造。

## MaterialBinding

被工程绑定的素材在目标源码中使用唯一精确 ABI：

```veac
struct MaterialBinding {
  kind: text, path: text, sha256: text, authority: text,
  artifact_key: text, video_stream: int, audio_stream: int,
}
input material hero: MaterialBinding;
```

`kind` 只能是 `video`、`audio`、`image`、`font`、`lut_1d`、`lut_3d`；`path` 是受 authority
约束的 canonical relative URI；`sha256` 是已验证内容身份。`authority` 只能是
`project_material` 或 `artifact`，后者必须携带 `artifact_key`。未选择的 stream 使用 `-1`。
这些字段全部进入 typed input digest；目标源码用 `hero.path` 和 `hero.sha256` 构造 resource，
而不是重新猜路径、扩展名或身份。

## Target 动作

`Veac` 对完整 source graph 执行 `main(Context) -> Project`，再经过 canonical IR、plan、typed
backend bundle 和 runtime。目标声明的每个 output 必须与 VEAC delivery 的 logical key 一一对应。

`MediaDerivation` 只接受以下闭合操作；表中参数全部是 authored、校验、cache identity 和 artifact
descriptor 的组成部分，不存在隐藏默认值。

| 操作 | 唯一输出 | 显式参数 |
| --- | --- | --- |
| `ProxyVideo` | video | `source`、`source_stream`、`source_clock`、`width`、`height`、`frame_rate`、`crf` |
| `ProxyAudio` | audio | `source`、`source_stream`、`source_clock`、`sample_rate`、`channels` |
| `Thumbnail` | image | `source`、`source_stream`、`at`、`width`、`height` |
| `Waveform` | image | `source`、`source_stream`、`source_clock`、`sample_rate`、`width`、`height`、`color` |
| `OpticalFlow` | video | `source`、`source_stream`、`source_clock`、`width`、`height`、`frame_rate`、`method` |
| `SourceSegment` | video | `source`、`video_stream`、`start`、`duration`、`width`、`height`、`frame_rate`、`audio`、`crf` |

`source` 必须绑定一个 `ProjectMaterial` 或单一 artifact。stream 同时携带 global/type index；
clock 只能是带 duration 的 `Identity` 或带 start/duration 的 `Bounded`；光流 method 只能是
`BlockMatching` 或 `MotionCompensated`；segment audio 显式给出 stream、sample rate 和 channels。

`Evidence` 运行 `evidence() -> EvidenceSuite`，要求恰好一个 directory output。完整合同见
[证据与验收](evidence.md)。

## 构建、缓存与交付

```bash
veac project check project.veac --package-root ../packages/components
veac project inspect project.veac --package-root ../packages/components
veac project graph project.veac --package-root ../packages/components
veac project build project.veac --package-root ../packages/components \
  --receipt build/receipt.json
```

`--package-root` 可重复，且是唯一 package 发现入口。CLI 先验证每个 manifest/lock/API trust loop，
再按 exact `name@version` 排序并拒绝重复 identity、互相重叠的 root，以及同一闭包 identity 对应
不同 content/API 合同的 split-brain 集合。工程 manifest、每个 target source graph、backend
identity 和 backend execution 始终复用同一个显式集合；缺少 root 不会回退到环境 store、网络或
当前目录发现。

宿主 package 路径只保存在本次进程的 `ProjectPackageSet`，不会写入 action 或 cache identity。
action v6 为每个 mount 固定 exact identity、package-relative entry、entry/content/API SHA-256、
排序的直接依赖和完整 locked closure，因此移动同内容 package 不会改变 action，内容、API 或闭包
变化则必然使 action 或执行校验失败。规划前后、loader 构造前后、backend identity 前后和执行前后
都会重做 discovery；计划完成后的漂移会在 cache lookup 之前失败。

调度器只运行依赖已满足且资源预算允许的节点。action、complete source graph revision、typed inputs、
上游 artifact 和实现版本共同形成 cache key。跨进程 computation lease 保证同一 key 只有一个 owner；
其他进程等待后重新读取 cache。receipt 固定记录 manifest/graph digest、执行或 cache-hit 状态、artifact
身份、交付状态和失败原因。显式 receipt 只能写入 `build_root`，并对 workspace、目标源码图和素材输入
执行同一套路径与 hard-link alias 防护，不能覆盖 authored source of truth。

所有输出先验证并发布到 `cache_root` 下的 content-addressed store，再从 CAS 原子交付。directory output
会先打包成确定性的 `VEACDIR1` artifact；解包时重新校验排序、路径、大小和摘要。已存在的同内容目录
可幂等复用，不同内容目录不可静默覆盖。staging 可删除，CAS 和 receipt 才是可审计事实。

## 真源与安全边界

`.veac` source graph 是唯一 authored source of truth。canonical JSON 只是严格版本化、可重建的 IR、
resolved graph 或 receipt，不应手写回工程。action 同时记录两种身份：authored revision 只覆盖
`Project` authority 的可编辑模块和清单；complete revision 覆盖 root、所有可达源码字节、authority 与
`(importer, requested, resolved)` route。package、builtin prelude 等只读依赖不进入编辑和 receipt 的
路径清单，但其 portable package revision 会直接进入每个 computation。backend 执行前重新准备
source graph 并核对 authored/complete revision 与 package revision，防止检查与执行之间被替换，
也不把宿主 ABI 或机器绝对路径伪装成用户源码。

工程动作不接受任意 shell、argv、FFmpeg filter 字符串或 property bag。它们会绕过类型检查、资源预算、
内容身份、cache key、平台一致性和输出验证。新能力必须先成为闭合的 nominal variant，并在 planner、
backend、artifact descriptor、验证与测试中使用同一合同。
