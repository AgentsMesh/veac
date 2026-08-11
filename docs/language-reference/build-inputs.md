# 声明式 Build input

`.veac` 不从 ambient 环境读取构建参数。root entry 必须静态声明每个 host input 的稳定名字、闭合
类型与 provenance role；module 不能声明或隐式注入 input。当前 role 是：

- `parameter`：用户或调用方显式选择的构建参数；
- `asset_metadata`：由受信 host 探测并显式绑定的素材事实；
- `analysis`：由分析流程生成并显式绑定的结果；
- `material`：由 project material root 或上游 Artifact DAG 显式授权的素材。

支持的纯叶类型是 `bool`、`int`、`scalar`、`text`、`time`、`length`、`angle`、`color`，以及不带
payload 的 nominal enum。enum 可以声明在 root entry，也可以从 module 导入；普通 role 的 struct、带
payload variant 的 enum 和 collection 不能作为 Build input。`material` role 是唯一例外，只接受字段名、
顺序和类型都精确匹配的 `MaterialBinding`。它们进入 typed HIR 时已经解析为 slot；verified
Core 保存稳定 `CoreBuildInputId` 与精确类型。runtime 按该 identity 绑定，不按字符串反射，也不接受
`map<text, Value>` property bag。

```veac,fragment
input parameter include_intro: bool;
input asset_metadata source_duration: time;
input analysis brand_color: color;

struct MaterialBinding {
  kind: text,
  path: text,
  sha256: text,
  authority: text,
  artifact_key: text,
  video_stream: int,
  audio_stream: int,
}
input material portrait: MaterialBinding;

enum Locale { zh-Hans, en, ja, }
input parameter locale: Locale;

fn intro_duration() -> time {
  if include_intro { source_duration } else { 0s }
}

fn main(context: Context) -> Project {
  let duration = intro_duration();
  let tint = brand_color;
  project(identifier("input-demo"), project_settings(600))
}
```

上面的 fragment 展示参数、素材元数据和分析结果如何进入普通纯函数与 graph build；`Context` 只代表
host 创建的 graph-local owner，不承载任意 key/value。

host 通过 versioned、deny-unknown JSON manifest 或等价 Rust API 提供值：

```json
{
  "schema": "https://veac.dev/schemas/build-inputs",
  "schema_version": 1,
  "inputs": [
    {"name":"include_intro","value":{"type":"bool","value":true}},
    {"name":"source_duration","value":{"type":"time","value":"4.25s"}},
    {"name":"brand_color","value":{"type":"color","value":"#1f7ae0ff"}},
    {"name":"locale","value":{"type":"enum","value":"zh-Hans"}},
    {"name":"portrait","value":{"type":"material","kind":"image","path":"portraits/hero.png","sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","authority":{"type":"project_material"},"video_stream":null,"audio_stream":null}}
  ]
}
```

数值使用确定性的十进制字符串。`time` 必须使用 `s`、`ms` 或 `us`，`length` 必须使用 `px`，
`angle` 必须使用 `deg`；`NaN`、无穷、错误单位和越界精度不进入 runtime。manifest 的未知字段、
错误 schema version、重复 binding、未知名字、缺失值和类型不一致都会在 graph transaction 发布前
失败。manifest 最大 16 MiB、最多 256 个 binding，单个 `text` 值最大 1 MiB；CLI 在读取前即执行
同一字节上限，Rust API 在绑定阶段执行相同的值约束。

`material.path` 必须是无 `.`、`..`、反斜杠、盘符或控制字符的 canonical 相对路径；`sha256` 必须是
64 位小写十六进制。`authority.type=project_material` 表示路径由项目 `material_root` 解析；
`authority.type=artifact` 还必须携带 64 位小写十六进制 `artifact_key`，表示值来自已验证 CAS artifact。
runtime struct 把未指定 stream 归一为 `-1`，并把 authority 归一为 `project_material` 或 `artifact`。
路径、content digest、authority、artifact key 和 stream selector 全部进入 Build-input identity。

`enum` manifest value 只携带 variant 名，不重复携带类型名。binder 必须先找到同名 input 声明，再按
该声明的 nominal `TypeId` 和 verified enum layout 精确构造值；未知 variant、nominal struct、带
payload 的 enum、primitive/enum tag 混用都会 fail closed。schema version 1 的 value union 明确包含
`enum`；host 必须按当前 schema 生成 manifest，不提供旧合同兼容或隐式迁移。

```bash
veac build main.veac --inputs build-inputs.json --emit-ir project.json
veac check main.veac --input locale=zh-Hans --input source_duration=4.25s \
  --input include_intro=true --input brand_color=#1f7ae0ff
veac source-index main.veac
veac source-edit main.veac edit.json --inputs build-inputs.json --input locale=ja --dry-run
veac schema --contract build-inputs
```

`--input NAME=VALUE` 可重复，适合 shell、CI matrix 和 Agent 只覆盖少量 locale 参数。CLI 先完成整个
source graph 的 prepare，再用同名声明的 expected type 解码 value；它不会把 `true`、数字或名字按外形
猜成某种类型。赋值只在第一个 `=` 处分隔，所以 `--input 'title=第一段=第二段'` 的 text value 会完整保留
后两个 `=`。manifest 与 inline 可以同时出现：`--inputs` 是 base，inline 按 name 覆盖；inline 自身的
重复 name 不采用 last-write-wins，而是失败。未知、缺失、类型错误、非法单位与 enum variant 同样
fail closed。`material` 包含多字段 authority 与 content identity，只允许 typed manifest 或 project
resolver 生成，不能用 `--input NAME=VALUE` 压扁成字符串。

text input 上限是 1 MiB UTF-8 bytes，不是 Unicode code point 数。生成的 JSON Schema 使用
`x-veac-max-utf8-bytes` 发布这个精确 byte 合同，不用语义不同的标准 `maxLength` 冒充；中文、阿拉伯文
和组合字符都按编码后的实际字节数执行同一限制。binding name 与 enum variant 必须满足 VEAC 的
1..128 ASCII byte declaration-name 合同。

`source-index` 不执行 graph，因此不接收 binding。它直接发布 Build-input signature；payloadless enum
包含合法 variants、nominal TypeId 和 definition digest，调用方无需预先猜一个 locale 才能发现接口。

最终 binding 先归一化为声明 identity 对应的 typed value，再计算 digest。因此参数排列顺序、值来自
manifest 还是 inline，以及 `2s`/`2000ms`、颜色十六进制大小写等表示差异都不会制造不同 provenance。
text 不做 trim 或 shell unescape；空格和 `=` 是否属于值由调用方的 shell quoting 明确决定。

声明合同和值都参与 executable identity 的 input digest；digest 使用单位换算和颜色大小写归一化后的
typed value，因此 `2s` 与 `2000ms` 具有相同 provenance，不同值则不同。enum digest 同时覆盖 nominal
type identity、definition digest 和 variant index；variant 或 enum 定义变化都会改变 executable identity。
Rust 调用方使用 `BuildInputManifestV1`、`BuildInputBinding` 和 `BuildInputManifestValue` 构造合同，并调用
`ExecutableBuild::execute_with_inputs` 或 `build_*_with_inputs`。
