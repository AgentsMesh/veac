# 证据与验收

Evidence 把“看起来应该正确”变成可执行、可缓存、可追溯的工程合同。手写入口是
`evidence() -> EvidenceSuite`；runtime 从已授权媒体采样或完整解码，纯 evaluator 计算断言，最后发布
一个 typed evidence bundle。断言失败也是需要保留的工程事实，不会因为退出状态而被删除。

```text
evidence.veac + imports
  -> validated EvidenceSuite
  -> deterministic ObservationPlan
  -> identity-bound media observations
  -> pure assertion evaluation
  -> report + provenance + review assets
  -> EvidenceBundle directory artifact in verified CAS
```

## EvidenceSuite

`EvidenceSuite` 等宿主类型由 `veac/evidence.veac` prelude 提供。合同由 source、sample、region 和
assertion 四组稳定 ID 构成；可复用 helper 和 imported module 仍使用普通 VEAC 函数与 nominal value。

```veac
fn between(minimum: scalar, maximum: scalar) -> OptionalRangeExpectation {
  OptionalRangeExpectation.Some { value: RangeExpectation {
    minimum: OptionalScalar.Some { value: minimum, },
    maximum: OptionalScalar.Some { value: maximum, },
  }, }
}

fn evidence() -> EvidenceSuite {
  EvidenceSuite {
    schema_version: 1,
    id: identifier("preview"),
    sources: [EvidenceSource {
      id: identifier("rendered"),
      binding: EvidenceSourceBinding.BoundInput { input_id: identifier("rendered"), },
    }],
    samples: [
      EvidenceSample { id: identifier("before"), source_id: identifier("rendered"), at: 0s, },
      EvidenceSample { id: identifier("after"), source_id: identifier("rendered"), at: 1s, }
    ],
    regions: [EvidenceRegion {
      id: identifier("card"),
      space: EvidenceRegionSpace.Normalized { x: 0.1, y: 0.1, width: 0.8, height: 0.8, },
    }],
    assertions: [
      EvidenceAssertion.DecodeComplete {
        id: identifier("decode"), source_id: identifier("rendered"), minimum_frames: 1,
      },
      EvidenceAssertion.PixelDiff {
        id: identifier("changed"), left_sample_id: identifier("before"),
        right_sample_id: identifier("after"),
        region_id: OptionalIdentifier.Some { value: identifier("card"), },
        channels: DiffChannels.Rgba, change_threshold: 8,
        expectation: DiffExpectation {
          rmse: OptionalRangeExpectation.None, mae: OptionalRangeExpectation.None,
          maximum_delta: OptionalRangeExpectation.None,
          changed_fraction: between(0.1, 1.0),
        },
      }
    ],
  }
}
```

sample time 使用精确 rational time。region 可用 `[0, 1]` normalized 坐标或整数 pixel 矩形；所有
引用、范围、ID、坐标和 observation budget 都在媒体执行前验证。

## Source authority

模型有 `Deliverable`、`Artifact`、`BoundInput` 三种闭合 binding。工程工作区中的 evidence target
只接受 `BoundInput`：对应的 `ProjectInput` 必须绑定 `ProjectMaterial` 或上游 `Artifact`。例如：

```veac
ProjectInput {
  id: identifier("rendered"),
  source: ProjectInputSource.Artifact {
    target: TargetRef {
      target: identifier("render"), profile: OptionalIdentifier.None,
      selector: InstanceSelector.Same,
    },
    output: identifier("video"),
  },
}
```

工作区 evidence 在 delivery 前运行，因此 `Deliverable` 没有可依赖的 authority；直接写
`EvidenceSourceBinding.Artifact` 也缺少 target/output/instance DAG 边。两者都会明确失败，而不是猜测
文件名。`BoundInput` 让依赖、内容 SHA-256、artifact key 和 cache identity 使用同一条事实链。

## 断言集合

| 断言 | 验证的事实 |
| --- | --- |
| `DecodeComplete` | 完整解码、最少帧数和 decode error |
| `Alpha` | alpha 均值及 transparent/partial/opaque 比例范围 |
| `PixelDiff` | RGB/RGBA/alpha 的 RMSE、MAE、最大差值和变化比例 |
| `Bounds` | alpha、difference 或 luma mask 的非空与边界尺寸 |
| `LayerOrder` | 两个 entity 共存时的前后顺序 |
| `CompositeOver` | actual 是否更接近正确的 source-over 合成结果 |
| `RevealOrder` | 多个区域是否按 checkpoint 的前缀顺序出现 |
| `MotionProfile` | changed fraction 或 alpha centroid 的运动量与减速曲线 |

每个 expectation 都使用 closed field 和 typed optional/range。没有表达式字符串、任意 metric 名或
property bag。backend 无法取得某项所需 observation 时，该断言产生 `error`，不会伪造成 `fail` 或
`pass`。

## Evidence target 与 bundle

工程 target 使用 `ProjectTargetEntry.Evidence { contract: "evidence.veac" }`，且必须声明恰好一个
`ProjectOutput.Directory`。执行成功后目录先编码为 deterministic `VEACDIR1`，再进入 verified CAS；
artifact descriptor 使用闭合的 `EvidenceBundle` variant，并绑定 suite SHA-256、report SHA-256 和
`pass | fail | error` outcome。

bundle 至少包含：

| 文件 | 内容 |
| --- | --- |
| `bundle.json` | schema、suite/provenance/report digest、cache key、outcome 和全部 payload 清单 |
| `suite.json`、`observation-plan.json` | 已验证合同及确定性采样计划 |
| `provenance.json` | source identity、stream、FFmpeg fingerprint、manifest 和 target instance |
| `observations.json` | 实际 PTS、decode 结果、layer order 和 observation failure |
| `results.json`、`results.csv`、`junit.xml` | 机器可读断言结果与 CI 投影 |
| `index.html`、`frames/*.png`、`contact-sheet.png` | 面向人工复核的预览；contact sheet 按需生成 |

每个 payload 的路径、大小和 SHA-256 都写入 manifest；发布前、CAS 读取和 directory delivery 时都会
重新验证。destination 已存在且内容不同、symlink、路径逃逸、乱序/重复 entry、损坏 payload 和 trailing
bytes 都会 fail closed。

## 命令与退出语义

```bash
veac project build project.veac --receipt build/build.json
veac project evidence project.veac --receipt build/evidence.json
veac project test project.veac --receipt build/test.json
```

| 命令 | 行为 |
| --- | --- |
| `project build` | 构建所有 target；不额外用 assertion outcome 做 gate |
| `project evidence` | 要求至少一个 typed evidence bundle；`fail` 或 `error` 仍成功发布 |
| `project test` | 先完成 bundle、CAS、delivery 和 receipt，再对任何 `fail`/`error` 返回 `PROJECT_TEST_FAILED` |

没有 evidence target 返回 `PROJECT_EVIDENCE_MISSING`；节点输出不是 typed bundle 返回
`PROJECT_EVIDENCE_CONTRACT`。三条命令的真实构建失败都返回 `PROJECT_BUILD_*`，不能被 assertion
语义吞掉。

需要区分三种结果：断言条件不满足是 `fail`；缺少可评价 observation 等可报告问题是 `error`；源码无效、
binding 不成立、执行取消、CAS 或 publication 失败是 build execution error。前两者拥有完整 bundle，
后者不会伪造一个“有效证据”目录。

## 可重现性与真源

cache identity 包含完整 evidence source graph、suite、observation plan、输入内容身份、producer
fingerprint、工程 manifest 和 target instance。root 不变但 imported helper 改变时也必须 cache miss。
runtime 在 observation 前后重新读取并核对 root、module inventory 和 graph SHA-256，阻止 TOCTOU；
宿主 builtin prelude 不计入用户 source revision。

`.veac` 仍是 authored source of truth。bundle 中的 JSON 是严格版本化的 observation/report IR 和审计
结果，不是下一轮手写入口。机器客户端可用以下命令取得闭合 schema：

```bash
veac schema --contract evidence-suite
veac schema --contract observation-plan
veac schema --contract evidence-report
veac schema --contract evidence-bundle
veac schema --contract evidence-provenance
```

Evidence 不执行任意 shell、argv 或 FFmpeg filter。媒体观察只能来自 typed request 和已验证 authority，
从而让断言、缓存、资源上限、producer fingerprint 与安全边界保持可审计。
