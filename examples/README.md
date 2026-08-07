# VEAC 示例

每个目录都以 `examples/<id>/main.veac` 作为唯一入口，并可包含入口导入的本地 `.veac` 模块。示例目录以画廊目录为唯一事实来源，用于发现示例、确定预览顺序、记录能力证据、指定预览时间窗口以及声明预期交付物：

```text
examples/catalog/gallery.json
examples/catalog/mechanisms/*.json
examples/capabilities.json
```

目录与文件系统必须包含完全相同的入口集合，不再保留旧版示例前端。`gallery.json.examples` 为每个入口维护一个标题、一段摘要，以及一组非空的 `{ cue, expect }` 检查项。观察点可以指向媒体时间、音频、规范中间表示、渲染计划或成品；说明文字不得把不可见的机制描述成像素证据。

## 常用命令

```bash
make check-examples
make build-examples
make serve-examples
make clean-examples
```

`check-examples` 会对目录中的每个入口及其完整源码图执行以下流程：

```text
解析 -> 格式化 -> 再次解析 -> 验证格式化幂等性
     -> 降级为规范 JSON 中间表示
     -> 执行规范校验
```

它还会校验能力元数据、画廊目标归属、工作流证据、入口注册唯一性，以及源码图的本地模块路径与符号链接安全约束。
格式化检查在内存快照上覆盖全部 38 个入口和所有可达模块，校验注释与非 trivia token 保真、二次格式化幂等，并用格式化后的完整源码图重新执行构建；该过程不会写回仓库示例。

`build-examples` 会把目录中的全部示例编译并渲染到已忽略的 `examples-preview/` 目录。构建过程会在该目录中准备生成式测试媒体和确定性预览字体，使用真实 CLI 与 FFmpeg 完成计划和渲染，验证声明的交付物，并生成 `examples-preview/index.html`。预览交付默认将最长边限制为 480 像素；可以通过 `PREVIEW_MAX_EDGE` 测试其他交付尺寸。

每个构建条目采用以下固定布局；旧的 `project.raw.json`、根级 `plan.json` 和 `plan.out_*.json` 不再兼容：

```text
examples-preview/<id>/
  project/
    main.veac
    <本地模块>.veac
    source.revision.json          # 可选的源码编辑证据
    source.index.json             # 可选的语义寻址清单
    source-edit.json              # 可选的原子编辑批次
    source-edit.outcome.json      # 可选的无写入预演结果
    project.veac.json
    project.preview.veac.json
    assets/
  plans/preview/<config-id>.json
  rendered/
  build.log
```

`main.veac` 和它导入的模块都是仓库源码图的字节保真副本，也是唯一可编辑事实源。声明源码编辑证据的示例还会发布精确版本、语义寻址索引、生产格式编辑批次和无写入预演结果，并验证预演产生新版本但不改变源码图。`project.veac.json` 由完整源码图编译一次得到；结构化预览策略会在这份 canonical IR 上统一适配字体、尺寸、帧率和时间窗口，派生 `project.preview.veac.json`。所有预览计划和渲染只消费预览 IR，不生成或重新编译第二份 `.veac` 源码。发布前会重新派生预览 IR、核对双 IR 中的项目与交付身份、逐一匹配预览配置与计划，并确认创作源码图和 IR 在整个渲染过程中未被改写。

发布前还会核对本次选择的示例目录闭包、规范中间表示和渲染计划，并逐个验证视频尺寸、帧率、计划时长、音频编码、采样率、声道与覆盖时长。每条视频都必须在开头、中点和结尾样本中包含可见内容，并通过音视频全量解码；MP3、GIF、单帧图片、图像序列、字幕、音频分轨、监看图和 HLS 包另有对应的格式与内容证据。

预览可以使用较小的渲染配置来提高交付速度，但变换过程必须保留每个序列的空间画布。合成、文本和空间参数都先按创作尺寸计算，最后才在输出适配阶段缩放画面；只有预览帧率可以在合成前降低。

构建过程不会改写规范关系事实、删除嵌入投影或注入新的交付语义；仓库中签入的示例始终保持不变。

## 能力覆盖

- `minimal`、`hello-world`：项目骨架，以及基础生成源和文本源。
- `programming-language`：跨模块 `struct`、闭合 `enum`、payloadless enum Build input、穷尽 `match` 和实例方法直接生成可见标题与配色，并结合 `let`、可观察的 `var/set`、显式 effect contract、嵌套 callable 的名义值与集合传递、`if/else`、短路逻辑、私有辅助函数、静态序列组件，以及以完整局部语句、函数体和方法体为源码事实源的带版本编辑。
- `executable-local-image`：函数创建 Project-owned 本地图片 Resource，媒体 Source 以非所有权方式引用它；不可变 Transform 原语组合锚点、缩放、旋转、翻转和平移，并输出可见中文确认标签。
- `timeline-source-time`、`speed-demo`：裁剪、变速、曲线和定格映射。
- `nested-and-multicam`：嵌套序列和类型化多机位切换程序。
- `generated-graphics`：透明、静音、纯色、渐变和图形源。
- `transforms-and-animation`、`blend-modes`、`masks-and-mattes`：视觉处理管线。
- `video-effects`：模糊、锐化、调色、暗角、颗粒、色度抠像、溢色抑制和亮度抠像效果栈。
- `card-overlay`、`image-overlay`：类型化卡片表面与可定位的图像叠加层。
- `apply-scopes`：`CompositeBand`、`Layer`、精确 `ItemSet`、有序阶段栈和 `Apply` 混合蒙版。
- `transitions`、`transition-gallery`：基于规范关系的转场。
- `audio-processing`、`executable-mechanisms`：处理器、路由、分组和音视频链接。
- `color-grade`、`advanced-color`：有序调色阶段和 LUT 引用。
- `text-overlay`、`text-layout`、`text-animation`：结构化文本值对象。
- `captions-and-sidecars`：字幕源、说话人、样式和伴随文件选择。
- `template-fill`：条目所属的媒体和文本模板插槽。
- `delivery-formats`：视频、图像序列、字幕边车、音频分轨、监看图、MP3、GIF、单帧和 HLS 九种类型化交付。
- `all-features`、`agentsmesh-intro-15s`：跨机制集成项目。

每一条能力记录都应指向精确的画廊目标和时间窗口。源码中出现某个词元并不能视为机制证据；集成测试必须能够把源码降级为对应的类型化中间表示。
