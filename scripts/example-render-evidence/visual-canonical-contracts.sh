#!/usr/bin/env bash

color_grade_canonical_contract() {
  jq -e '
    def seconds: .value/.timescale;
    def clips: reduce (.project.sequences[].tracks[].clips[]) as $clip
      ({}; .[$clip.authorship.logical_path[-1]]=$clip);
    clips as $c |
    def centered: {type:"anchor",anchor:"center",inset:{x:0,y:0}};
    ($c.reference.record_range.start|seconds)==0 and
    ($c.reference.record_range.duration|seconds)==2 and
    ($c.balanced.record_range.start|seconds)==2 and
    ($c.balanced.record_range.duration|seconds)==2 and
    $c.reference.source==$c.balanced.source and
    $c.reference.source.generator.gradient.type=="linear" and
    $c.reference.visual.color_pipeline==null and
    $c.balanced.visual.placement==centered and
    $c.balanced.visual.transform.anchor=={x:0.5,y:0.5} and
    $c.balanced.visual.color_pipeline as $pipeline |
    $pipeline.input=={"matrix":"bt709","primaries":"bt709","range":"limited","transfer":"bt709"} and
    $pipeline.working=={"matrix":"rgb","primaries":"bt709","range":"full","transfer":"linear"} and
    $pipeline.output==$pipeline.input and
    $pipeline.stages==[{"type":"basic","adjustment":{"exposure_stops":0.4,
      "fade":0.04,"highlights":-0.18,"shadows":0.28,
      "temperature_kelvin":4800,"tint":0.08}}] and
    $c["reference-label"].source.text=="原始色彩参照" and
    $c["grade-label"].source.text=="曝光、色温、色调、高光、阴影与褪色" and
    ($c["reference-label"].record_range.start|seconds)==0 and
    ($c["grade-label"].record_range.start|seconds)==2
  ' "$1" >/dev/null || fail "color-grade canonical mechanism identity contract failed"
}

blend_modes_canonical_contract() {
  jq -e '
    def seconds: .value/.timescale;
    def clips: reduce (.project.sequences[].tracks[].clips[]) as $clip
      ({}; .[$clip.authorship.logical_path[-1]]=$clip);
    clips as $c |
    def centered: {type:"anchor",anchor:"center",inset:{x:0,y:0}};
    ["screen","multiply","overlay","darken","lighten","color-dodge","color-burn",
     "hard-light","soft-light","difference","exclusion","normal"] as $keys |
    ["screen","multiply","overlay","darken","lighten","color_dodge","color_burn",
     "hard_light","soft_light","difference","exclusion","normal"] as $modes |
    ["screen-label","multiply-label","overlay-label","darken-label","lighten-label",
     "dodge-label","burn-label","hard-label","soft-label","difference-label",
     "exclusion-label","normal-label"] as $labels |
    ["滤色","正片叠底","叠加","变暗","变亮","颜色减淡","颜色加深","强光","柔光",
     "差值","排除","正常参照"] as $texts |
    ($c.background.record_range.duration|seconds)==12 and
    $c.background.source.generator.gradient.type=="linear" and
    all(range(0;12); . as $index | $c[$keys[$index]] as $clip |
      ($clip.record_range.start|seconds)==$index and
      ($clip.record_range.duration|seconds)==1 and
      $clip.source.generator.gradient.type=="radial" and
      $clip.visual.placement==centered and
      $clip.visual.transform.anchor=={x:0.5,y:0.5} and
      $clip.visual.compositing.blend_mode==$modes[$index] and
      $clip.visual.opacity=={"type":"constant","value":0.82} and
      ($c[$labels[$index]].record_range.start|seconds)==$index and
      ($c[$labels[$index]].record_range.duration|seconds)==1 and
      $c[$labels[$index]].source.text==$texts[$index])
  ' "$1" >/dev/null || fail "blend-modes canonical mechanism identity contract failed"
}

apply_scopes_canonical_contract() {
  jq -e '
    def seconds: .value/.timescale;
    def clip($key): first(.project.sequences[].tracks[].clips[] |
      select(.authorship.logical_path[-1]==$key));
    def owner($id): first(.project.sequences[].tracks[] | select(any(.clips[]; .id==$id)));
    clip("background") as $background | clip("accent") as $accent |
    {type:"anchor",anchor:"center",inset:{x:0,y:0}} as $centered |
    owner($background.id) as $base | owner($accent.id) as $overlay |
    ([.project.sequences[].applies[]]|sort_by(.record_range.start|seconds)) as $a |
    ($a|length)==3 and
    $accent.visual.placement==$centered and
    $accent.visual.transform.anchor=={x:0.5,y:0.5} and
    ($a|map([(.record_range.start|seconds),(.record_range.duration|seconds)]))==
      [[0,2],[2,2],[4,2]] and
    $a[0].target=={"type":"composite_band","from_track_id":$base.id,
      "through_track_id":$overlay.id} and
    ($a[0].stages|map(.operation.effect.effect.type))==
      ["video_color_adjust","video_blur"] and
    $a[0].stages[0].operation.effect.effect.contrast.value==1.3 and
    $a[0].stages[1].operation.effect.effect.radius.value==14 and
    $a[0].mix.blend_mode=="screen" and $a[0].mix.opacity.value==0.88 and
    ($a[0].mix.masks|length)==1 and
    $a[1].target=={"type":"layer","track_id":$overlay.id} and
    $a[1].stages[0].operation.effect.effect.type=="video_color_adjust" and
    $a[1].stages[0].operation.effect.effect.brightness.value==0.35 and
    $a[2].target=={"type":"item_set","item_ids":[$background.id,$accent.id]} and
    $a[2].stages[0].operation.effect.effect.contrast.value==1.55 and
    $a[2].mix.blend_mode=="multiply" and $a[2].mix.opacity.value==0.7 and
    clip("band-label").source.text=="作用域：合成时间带" and
    clip("layer-label").source.text=="作用域：单个图层" and
    clip("items-label").source.text=="作用域：精确条目集合"
  ' "$1" >/dev/null || fail "apply-scopes canonical mechanism identity contract failed"
}
