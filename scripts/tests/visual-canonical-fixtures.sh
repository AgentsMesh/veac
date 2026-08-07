#!/usr/bin/env bash

write_color_grade_fixture() {
  jq -n '
    def t($v): {timescale:1,value:$v};
    def path($key): {logical_path:["color-grade",$key]};
    def centered: {placement:{type:"anchor",anchor:"center",inset:{x:0,y:0}},
      transform:{anchor:{x:0.5,y:0.5}}};
    {type:"generated",generator:{type:"gradient",gradient:{type:"linear"}}} as $source |
    {project:{sequences:[{tracks:[{clips:[
      {id:"reference",authorship:path("reference"),record_range:{start:t(0),duration:t(2)},
       source:$source,visual:{color_pipeline:null}},
      {id:"balanced",authorship:path("balanced"),record_range:{start:t(2),duration:t(2)},
       source:$source,visual:(centered+{color_pipeline:{
         input:{matrix:"bt709",primaries:"bt709",range:"limited",transfer:"bt709"},
         working:{matrix:"rgb",primaries:"bt709",range:"full",transfer:"linear"},
         output:{matrix:"bt709",primaries:"bt709",range:"limited",transfer:"bt709"},
         stages:[{type:"basic",adjustment:{exposure_stops:0.4,fade:0.04,
           highlights:-0.18,shadows:0.28,temperature_kelvin:4800,tint:0.08}}]}})},
      {authorship:path("reference-label"),record_range:{start:t(0),duration:t(2)},
       source:{text:"原始色彩参照"}},
      {authorship:path("grade-label"),record_range:{start:t(2),duration:t(2)},
       source:{text:"曝光、色温、色调、高光、阴影与褪色"}}]}]}]}}
  ' >"$1"
}

write_blend_modes_fixture() {
  jq -n '
    def t($v): {timescale:1,value:$v};
    def path($key): {logical_path:["blend-modes",$key]};
    def centered: {placement:{type:"anchor",anchor:"center",inset:{x:0,y:0}},
      transform:{anchor:{x:0.5,y:0.5}}};
    ["screen","multiply","overlay","darken","lighten","color-dodge","color-burn",
     "hard-light","soft-light","difference","exclusion","normal"] as $keys |
    ["screen","multiply","overlay","darken","lighten","color_dodge","color_burn",
     "hard_light","soft_light","difference","exclusion","normal"] as $modes |
    ["screen-label","multiply-label","overlay-label","darken-label","lighten-label",
     "dodge-label","burn-label","hard-label","soft-label","difference-label",
     "exclusion-label","normal-label"] as $labels |
    ["滤色","正片叠底","叠加","变暗","变亮","颜色减淡","颜色加深","强光","柔光",
     "差值","排除","正常参照"] as $texts |
    (reduce range(0;12) as $i
      ([{authorship:path("background"),record_range:{start:t(0),duration:t(12)},
         source:{generator:{gradient:{type:"linear"}}}}];
       .+[{authorship:path($keys[$i]),record_range:{start:t($i),duration:t(1)},
           source:{generator:{gradient:{type:"radial"}}},
           visual:(centered+{compositing:{blend_mode:$modes[$i]},
             opacity:{type:"constant",value:0.82}})},
          {authorship:path($labels[$i]),record_range:{start:t($i),duration:t(1)},
           source:{text:$texts[$i]}}])) as $clips |
    {project:{sequences:[{tracks:[{clips:$clips}]}]}}
  ' >"$1"
}

write_apply_scopes_fixture() {
  jq -n '
    def t($v): {timescale:1,value:$v};
    def path($key): {logical_path:["apply-scopes",$key]};
    def centered: {placement:{type:"anchor",anchor:"center",inset:{x:0,y:0}},
      transform:{anchor:{x:0.5,y:0.5}}};
    def color($brightness;$contrast): {operation:{effect:{effect:{type:"video_color_adjust",
      brightness:{value:$brightness},contrast:{value:$contrast}}}}};
    {project:{sequences:[{tracks:[
      {id:"base",clips:[{id:"background",authorship:path("background")}]},
      {id:"overlay",clips:[{id:"accent",authorship:path("accent"),visual:centered}]},
      {id:"labels",clips:[
        {authorship:path("band-label"),source:{text:"作用域：合成时间带"}},
        {authorship:path("layer-label"),source:{text:"作用域：单个图层"}},
        {authorship:path("items-label"),source:{text:"作用域：精确条目集合"}}]}],
      applies:[
        {record_range:{start:t(0),duration:t(2)},target:{type:"composite_band",
          from_track_id:"base",through_track_id:"overlay"},
         stages:[color(0;1.3),{operation:{effect:{effect:{type:"video_blur",radius:{value:14}}}}}],
         mix:{blend_mode:"screen",opacity:{value:0.88},masks:[{}]}},
        {record_range:{start:t(2),duration:t(2)},target:{type:"layer",track_id:"overlay"},
         stages:[color(0.35;1)],mix:{blend_mode:"normal",opacity:{value:1},masks:[]}},
        {record_range:{start:t(4),duration:t(2)},target:{type:"item_set",
          item_ids:["background","accent"]},stages:[color(0;1.55)],
         mix:{blend_mode:"multiply",opacity:{value:0.7},masks:[]}}]}]}}
  ' >"$1"
}
