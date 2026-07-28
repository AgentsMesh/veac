# VEAC 示例

每个目录都在 `examples/<id>/main.veac` 中包含一份规范创作源码。示例目录以画廊目录为唯一事实来源，用于发现示例、确定预览顺序、记录能力证据、指定预览时间窗口以及声明预期交付物：

```text
examples/catalog/gallery.json
examples/catalog/mechanisms/*.json
examples/capabilities.json
```

目录与文件系统必须包含完全相同的源码集合，不再保留旧版示例前端。`gallery.json.examples` 为每个源码维护一个标题、一段摘要，以及一组非空的 `{ cue, expect }` 检查项。观察点可以指向媒体时间、音频、规范中间表示、渲染计划或成品；说明文字不得把不可见的机制描述成像素证据。

## 常用命令

```bash
make check-examples
make build-examples
make serve-examples
make clean-examples
```

`check-examples` 会对目录中的每份源码执行以下完整流程：

```text
解析 -> 格式化 -> 再次解析 -> 验证格式化幂等性
     -> 降级为规范 JSON 中间表示
     -> 执行规范校验
```

它还会校验能力元数据、画廊目标归属、工作流证据、目录注册唯一性，以及每个示例目录只能包含一份 `.veac` 源码。

`build-examples` 会把目录中的全部示例编译并渲染到已忽略的 `examples-preview/` 目录。构建过程会在该目录中准备生成式测试媒体和确定性预览字体，使用真实 CLI 与 FFmpeg 完成计划和渲染，验证声明的交付物，并生成 `examples-preview/index.html`。预览交付默认将最长边限制为 480 像素；可以通过 `PREVIEW_MAX_EDGE` 测试其他交付尺寸。

预览可以使用较小的渲染配置来提高交付速度，但变换过程必须保留每个序列的空间画布。合成、文本和空间参数都先按创作尺寸计算，最后才在输出适配阶段缩放画面；只有预览帧率可以在合成前降低。

构建过程不会改写规范关系事实，也不会删除嵌入投影。对于没有视频交付物的项目，它会在编译前向生成的 `.veac` 副本注入类型化预览输出，仓库中签入的示例保持不变。

## 能力覆盖

- `minimal`、`hello-world`：项目骨架，以及基础生成源和文本源。
- `timeline-source-time`、`speed-demo`：裁剪、变速、曲线和定格映射。
- `nested-and-multicam`：嵌套序列和类型化多机位切换程序。
- `generated-graphics`：透明、静音、纯色、渐变和图形源。
- `transforms-and-animation`、`blend-modes`、`masks-and-mattes`：视觉处理管线。
- `apply-scopes`：`CompositeBand`、`Layer`、精确 `ItemSet`、有序阶段栈和 `Apply` 混合蒙版。
- `transitions`、`transition-gallery`：基于规范关系的转场。
- `audio-processing`、`executable-mechanisms`：处理器、路由、分组和音视频链接。
- `color-grade`、`advanced-color`：有序调色阶段和 LUT 引用。
- `text-overlay`、`text-layout`、`text-animation`：结构化文本值对象。
- `captions-and-sidecars`：字幕源、说话人、样式和伴随文件选择。
- `template-fill`：条目所属的媒体和文本模板插槽。
- `delivery-formats`：全部五种类型化输出变体。
- `all-features`、`agentsmesh-intro-15s`：跨机制集成项目。

每一条能力记录都应指向精确的画廊目标和时间窗口。源码中出现某个词元并不能视为机制证据；集成测试必须能够把源码降级为对应的类型化中间表示。
