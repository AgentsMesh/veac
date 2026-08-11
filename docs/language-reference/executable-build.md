# 可执行 Build

VEAC 的 executable frontend 会真正执行程序来构造视频项目，不是把 Surface AST 直接改写成
JSON，也不是在 JavaScript 上套一层框架。当前生产路径是：

```text
.veac source graph
  -> Surface AST + module/name resolution
  -> typed HIR + effect/stage inference
  -> verified Core v10
  -> bounded evaluator + one graph transaction
  -> connected frozen domain graph
  + root animate declarations / component animate attachments
  -> direct canonical ProjectEnvelope + residual TemporalProgramLibrary
  -> canonical validation -> planner -> backend
```

runtime 只接收 verified Core。它不解释 Surface AST、不按源码字符串分派调用，也不会把 graph
重新渲染成 `.veac` 后交给 legacy parser。canonical JSON 仍是 backend、interchange、cache 与
validation ABI；`.veac` source graph 始终是 authoring source of truth。

## 完整程序

下面的代码使用当前真实语法，并覆盖 nominal value、method、穷尽匹配与直接 Project graph
construction：

```veac,executable
enum Tone { Calm, Accent { value: color, }, }
struct Card { key: identifier, at: time, title: text, tone: Tone, }

impl Card @presentation {
  fn fill(self) -> color {
    match self.tone {
      Tone.Calm => #245b78ff,
      Tone.Accent { value } => value,
    }
  }
  fn build(self, font: Resource) -> Item {
    title_item(self.key, self.title, self.at, self.fill(), font)
  }
}
fn title_item(key: identifier, title: text, at: time, fill: color, font: Resource) -> Item {
  let style = text_style(
    text_metrics(font_stack(font_resource_ref(font), []), weight_bold(), font_style_normal(), 48px, 0px, 1.2, fill),
    text_layout(text_box_width(480px), text_wrap_word(), text_overflow_clip(), text_align_center(), text_align_middle(), writing_horizontal_tb(), orientation_mixed()),
    text_path_none(), text_decoration(text_background_none(), text_outline_none(), shadow_none()), [], text_animation_none()
  );
  let visual = visual_style(
    visual_layout(placement_anchor(anchor_center(), vector(0.0, 0.0)), frame_none(), transform_2d(transform_motion(point_constant(point(24px, 0px)), vector_constant(vector(1.0, 1.0)), angle_constant(0deg)), transform_geometry(vector(0.0, 0.0), flip_none(), vector(0.5, 0.5), crop_none()))),
    visual_surface(percent_constant(100%), compositing(0, blend_normal()), card_none()), [], color_pipeline_none()
  );
  item(key, item_enabled(), during(at, 2s), source_text(title, style), source_timing_native()).with_visual(visual)
}

fn main(context: Context) -> Project {
  let font = font_resource(identifier("font"), resource_file("assets/font.ttf"), sha256("0000000000000000000000000000000000000000000000000000000000000000"));
  let first_card = Card { key: identifier("first"), at: 0s, title: "第一章", tone: Tone.Calm, };
  let second_card = Card { key: identifier("second"), at: 2s, title: "第二章", tone: Tone.Accent { value: #8d315bff, }, };
  let first = first_card.build(font);
  let second = second_card.build(font);
  let state = track_state(track_playback_enabled(), track_audio_audible(), track_isolation_normal(), track_editing_unlocked());
  let layer = visual_layer(identifier("titles"), 0, placement_free(), state, track_routing_default())
    .with_item(first).with_item(second);
  let timeline = sequence(identifier("main"), "主时间线", sequence_settings(canvas(1280px, 720px), frame_rate(30, 1), 48000)).with_layer(layer);
  project(identifier("guide"), project_settings(600))
    .with_resource(font)
    .with_sequence(timeline)
    .entry(timeline)
}
```

executable entry 必须在 root file 中恰好声明一个
`fn main(context: Context) -> Project`。参数名、参数类型和返回类型都是 ABI；imported `main`
不能充数，entry 也不能同时包含 legacy `project` block。host 在本次 transaction 内创建
graph-local `Context`，它不是可以伪造或跨 build 传递的普通 external value。

## 模块与复用

imported file 使用匿名 `module {}` marker，由 loader 的 root-relative source ID 定义身份：

```veac,fragment
module {
  export struct Chapter { key: identifier, title: text, }

  fn decorate(value: text) -> text { "章节：" + value }

  impl Chapter @timing {
    export fn label(self) -> text { decorate(self.title) }
  }
}
```

调用方以 `import "./chapter.veac" as chapter;` 选择 lexical alias。只有 `export` declaration
跨 module boundary；private helper 仍可被 exported function 或 method 调用。函数、method、closure、
struct、closed enum、exhaustive `match`、list/map/tuple/range 和 `if`/`for` 都先解析为 typed HIR，
随后按 numeric ID lower 到 Core。runtime 不进行动态名字查找、反射、继承或 prototype dispatch。

## Effect 与 Stage

两套推导彼此正交：

```text
Effect = Pure | LocalMutation | GraphEmit
Stage  = Const < Build < Temporal
```

当前 value/source description constructor 是 `Pure`；graph entity constructor 和 attachment 是
`GraphEmit`。`LocalMutation` 已是 closed Core effect，但当前 41 个 domain operation 没有公开这种
Surface 能力。Core 在最大 effect summary 外独立保留 mutation 证据，函数 effect 由实际调用推导。

Core 分别跟踪 aggregate `shape` 与 `leaf` stage。实体是否存在、key、kind、parent、order、集合
长度、source/resource 选择等 topology sink 必须不晚于 `Build`；只有被 operation contract 标记为
leaf 的媒体参数才可能进入 `Temporal`。核心不变量是 **static topology, dynamic leaf values**。

`map` 和由 Surface `for` 合成的 callback 可推导为 `GraphEmit`，并在同一 transaction 中生成有界
topology。每个实体仍必须给出显式 key，同一 parent 下重复 `(DomainType, key)` 或 ownership 会失败。
`filter` 与 `fold`
callback 仍必须是 `Pure`；所有 collection callback 都拒绝 `LocalMutation`。root authored declaration
与 component owner-relative attachment 都只允许 approved leaf residualize；完整合同见
[可执行 Temporal](executable-temporal.md)。

## Transform 代数

`transform()` 创建恒等值；`translated` 累加像素、`scaled` 逐轴相乘、`rotated` 累加角度、
`anchored` 后值替换前值、`flipped` 以异或切换轴向。每次调用返回独立描述，
`Item.with_transform` 则替换条目的整个变换值。它们不是任意仿射矩阵拼接：lowering 固定写入
`Transform2D` 的 anchor/frame、flip、scale、rotation、pixel-position 规范通道顺序，调用顺序只按
上述代数合并同名通道。scale 必须在 `(0, MAX_VISUAL_SCALE]`，anchor 必须在 `[0,1]`。

## Transaction 与 Freeze

一个顶层 evaluator 持有唯一 graph transaction；`main`、direct call、method 与 closure 共用它。
Domain handle 是 opaque、immutable 且 graph-affine 的值，不能作为 public input、literal、map key、
equality operand 或普通 evaluator 结果泄漏。attach 会检查 kind、single ownership、duplicate key、
stale/forged/cross-graph handle 和 cycle。

预算在 mutation 前原子预留。任何 runtime、budget、ownership、freeze、lowering 或 canonical
validation 错误都会丢弃整个 transaction，不能发布部分 graph、IR 或 source-edit preview。freeze
要求返回值是 closed、connected、且有合法 entry sequence 的 `Project`。canonical entity ID 由完整
owner logical path 和 entity kind 确定，因此 sibling 重排不改名，不同 parent 下同名 leaf 不碰撞。

`Relation` 由 Sequence 拥有，但只以稳定 Node identity 非拥有地引用两个 Item。当前
`transition(key, from, to, dissolve(duration))` 要求端点不同、最终位于同一 Sequence 的同一 Visual
Layer；Item 的后续 immutable update 不改变引用身份。lowering 生成 centered true-overlap transition，
duration 必须精确等于两端 record range 的交集，且不会通过冻结端点补帧伪造 overlap。

`sha256` 生成不能由普通 text 冒充的 `ContentIdentity`；图片、音频和字体 Resource 必须在源码中携带
64 位小写十六进制摘要。`text_style` 显式引用字体 Resource；Audio Item 只接受音频 media，Caption
Item 只进入 Caption Layer。默认 preview 交付包含双声道、48 kHz AAC，字幕按 raster 设置烧录。

## CLI 与唯一 Frontend

```bash
veac check main.veac
veac build main.veac --emit-ir project.json
veac check-ir project.json
```

`build`、`check`、`fmt` 与 `source-*` 只走 executable pipeline；没有自动分类、legacy fallback 或
`--frontend` 开关。含本地 Resource 时，`--emit-ir` 必须写在 source root，stdout consumer 也必须维持
同一素材基准。entry 必须定义 root-local `main`；legacy `project` block 或混合 entry 会失败。

## Source-Of-Truth Edit

```bash
veac source-index main.veac
veac source-edit main.veac edit.json --dry-run
```

Agent 通过 source-index v8 的 module-qualified semantic target 修改 expression、callable body、
declaration、import 或 typed anchor。source-edit v6 校验 revision 与 fragment，并以一个 transaction
重新执行完整 executable build；多模块任一环节失败都不写源码。不要修改生成的 canonical entity ID，
也不要从 JSON IR 反编译 `.veac`。

## 当前 Slice 与边界

当前 opset v8 固定 `214` 个 DomainType 和 `582` 个 numeric operation，覆盖 topology entity 与其闭合
value algebra。完整机器合同以 `veac language-spec` 为准；type/function/method 名是标准库 symbol，
不是 lexer keyword，不能计入关键字或 syntax spelling 数量。

每个机制仍必须以 family 为单位纵向闭合：DomainType/operation contract、Core verifier/runtime、
direct lowering、source edit 与测试同时落地。legacy authoring/expand 已从 production source 删除；
不能把 canonical JSON field 暴露成平铺 property block，也不能用字符串重解析绕过 typed Core。
