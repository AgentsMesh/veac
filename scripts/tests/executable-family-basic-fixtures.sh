#!/usr/bin/env bash

write_programming_fixture() {
  local root=$1 dir spec plan_spec
  dir=$(prepare_executable_fixture "$root" programming-language)
  spec="$dir/project.spec.json"; plan_spec="$dir/plan.spec.json"
  jq -n '
    def range($s;$d): {start:{timescale:600,value:$s},duration:{timescale:600,value:$d}};
    def path($track;$key): ["programming-language","sequence","main","layer",$track,"item",$key];
    def event($operation): {operation:$operation};
    def provenance($track;$key): {logical_path:path($track;$key),
      events:[{call_stack:[{function:"<closure>"}]}]};
    def backdrop($key;$id;$s;$color;$binding): {id:$id,authorship:provenance("backdrops";$key),
      record_range:range($s;1800),source:{type:"generated",generator:{type:"solid",color:$color}},
      visual:{opacity:{type:"binding",binding_id:$binding}}};
    def title($key;$id;$s;$text): {id:$id,authorship:provenance("titles";$key),
      record_range:range($s;1800),source:{type:"text",text:$text,
        style:{font:{type:"material",material_id:"med_font"}}}};
    {name:"可编程语言能力：结构与方法：静态类型组件 / 枚举与控制流：穷尽匹配生成",
      project_entity:{events:[event(8222),event(8234),event(8235),event(8231),event(8238)]},
      sequence_entity:{events:[event(8223),event(8239)]},
      track_authorship:[
        {track_id:"trk_backdrops",entity:{events:[event(8226),event(8242)]}},
        {track_id:"trk_titles",entity:{events:[event(8226),event(8242)]}}],
      materials:[{id:"med_font",kind:"font",source:{uri:"assets/veac-example-zh.ttf"},
        identity:{digest:"64afada094cf447d71fad4ee726799c4fa74917f7d9a1113ed7ea3c4234ff774"},
        authorship:{logical_path:["programming-language","resource","example-font"]}}],
      tracks:[{id:"trk_backdrops",clips:[
        backdrop("first";"itm_first";0;{red:43,green:101,blue:116,alpha:255};"tbd_first"),
        backdrop("second";"itm_second";1800;{red:123,green:49,blue:93,alpha:255};"tbd_second")]},
        {id:"trk_titles",clips:[
          title("first";"itm_first_title";0;"组件化：结构与方法：静态类型组件"),
          title("second";"itm_second_title";1800;"枚举与控制流：穷尽匹配生成")]}]}
  ' >"$spec"
  write_executable_project "$spec" "$dir/project/project.veac.json" \
    programming-language 1280 720 30
  copy_executable_preview "$dir/project/project.veac.json" "$dir/project/project.preview.veac.json"
  jq -n '
    def clip($id;$text;$opacity): {id:$id,source:{content:{text:$text}},visual:{opacity:$opacity}};
    {inputs:[{material_id:"med_font",canonical_uri:"assets/veac-example-zh.ttf",
      observed_identity:{digest:"64afada094cf447d71fad4ee726799c4fa74917f7d9a1113ed7ea3c4234ff774"}}],
      tracks:[{clips:[clip("itm_first";null;{type:"binding",binding_id:"tbd_first"}),
      clip("itm_second";null;{type:"binding",binding_id:"tbd_second"}),
      clip("itm_first_title";"组件化：结构与方法：静态类型组件";null),
      clip("itm_second_title";"枚举与控制流：穷尽匹配生成";null)]}],
      temporal:{bindings:[
        {program_id:"tpg_fade",clocks:[{owner:{type:"item",item_id:"itm_first"}}]},
        {program_id:"tpg_fade",clocks:[{owner:{type:"item",item_id:"itm_second"}}]}],
        programs:[{nodes:["input","literal","binary","literal","compare","literal","select",
          "literal","compare","literal","select","curve_sample"]|map({kind:{type:.}})}]}}
  ' >"$plan_spec"
  write_executable_plan "$plan_spec" "$dir/plans/preview/out_preview.json" 6 480 270 discard null
}

write_audio_caption_fixture() {
  local root=$1 dir spec plan_spec
  dir=$(prepare_executable_fixture "$root" executable-audio-caption)
  spec="$dir/project.spec.json"; plan_spec="$dir/plan.spec.json"
  jq -n '
    def range($s;$d): {start:{timescale:600,value:$s},duration:{timescale:600,value:$d}};
    def path($track;$key): ["executable-audio-caption","sequence","main","layer",$track,"item",$key];
    def caption($key;$id;$s;$d;$text;$speaker): {id:$id,authorship:{logical_path:path("captions";$key)},
      record_range:range($s;$d),source:{type:"caption",text:$text,speaker:$speaker,
        style:{font:{type:"material",material_id:"med_font"}}}};
    {materials:[
      {id:"med_tone",kind:"audio",source:{uri:"assets/tone.wav"},
        identity:{digest:"5c3aaa006e4c341feddf589f9eb0ba3b215c60353e491abf8f06f4bee4478cfc"},
        authorship:{logical_path:["executable-audio-caption","resource","tone"]}},
      {id:"med_font",kind:"font",source:{uri:"assets/veac-example-zh.ttf"},
        identity:{digest:"64afada094cf447d71fad4ee726799c4fa74917f7d9a1113ed7ea3c4234ff774"},
        authorship:{logical_path:["executable-audio-caption","resource","example-font"]}}],
      tracks:[{id:"trk_tone",clips:[{id:"itm_tone",authorship:{logical_path:path("tone";"tone")},
        record_range:range(600;1200),source:{type:"media",material_id:"med_tone"},audio:{
          gain:{type:"constant",value:.7},pan:{type:"constant",value:0},muted:false,normalize:false,
          pitch_policy:"preserve",processors:[],crossfade:{fade_in:{timescale:600,value:60},
            fade_out:{timescale:600,value:60},curve:"equal_power"}}}]},
        {id:"trk_captions",clips:[caption("intro";"itm_intro";0;900;"可执行音频与字幕";"小石"),
          caption("audible";"itm_audible";900;900;"有声音轨：均衡功率淡入淡出";null),
          caption("silent";"itm_silent";1800;600;"无声区间：字幕仍然保持";null)]}]}
  ' >"$spec"
  write_executable_project "$spec" "$dir/project/project.veac.json" \
    executable-audio-caption 640 360 24
  copy_executable_preview "$dir/project/project.veac.json" "$dir/project/project.preview.veac.json"
  jq -n '
    def range($s;$d): {start:{timescale:600,value:$s},duration:{timescale:600,value:$d}};
    def caption($id;$text;$speaker): {id:$id,source:{content:{text:$text,presentation:{style:{font:{requested:{material_id:"med_font"}}}}},speaker:$speaker}};
    {inputs:[{id:"pin_tone",material_id:"med_tone",
      observed_identity:{digest:"5c3aaa006e4c341feddf589f9eb0ba3b215c60353e491abf8f06f4bee4478cfc"},
      audio:{info:{sample_rate:48000,channels:1,channel_layout:"unknown"}}},
      {id:"pin_font",material_id:"med_font",observed_identity:{digest:
        "64afada094cf447d71fad4ee726799c4fa74917f7d9a1113ed7ea3c4234ff774"}}],
      tracks:[{clips:[{id:"itm_tone",record_range:range(600;1200),
        source:{type:"media",input_id:"pin_tone",audio_stream:{global_index:0,type_index:0}},
        audio:{crossfade:{curve:"equal_power"}}},
        caption("itm_intro";"可执行音频与字幕";"小石"),
        caption("itm_audible";"有声音轨：均衡功率淡入淡出";null),
        caption("itm_silent";"无声区间：字幕仍然保持";null)]}]}
  ' >"$plan_spec"
  write_executable_plan "$plan_spec" "$dir/plans/preview/out_preview.json" 4 480 270 burn_in \
    '{"codec":"aac","sample_rate":48000,"channels":2}'
}

write_dissolve_fixture() {
  local root=$1 dir spec plan_spec
  dir=$(prepare_executable_fixture "$root" executable-centered-dissolve)
  spec="$dir/project.spec.json"; plan_spec="$dir/plan.spec.json"
  jq -n '
    def range($s;$d): {start:{timescale:600,value:$s},duration:{timescale:600,value:$d}};
    def path($track;$key): ["executable-centered-dissolve","sequence","main","layer",$track,"item",$key];
    {materials:[],tracks:[{id:"trk_scenes",clips:[
      {id:"itm_a",authorship:{logical_path:path("scenes";"scene-a")},record_range:range(0;1200)},
      {id:"itm_b",authorship:{logical_path:path("scenes";"scene-b")},record_range:range(960;1440)}]},
      {id:"trk_labels",clips:[{id:"itm_mix",authorship:{logical_path:path("labels";"label-mix")},
        source:{type:"text",text:"居中叠化：红色与蓝色同时存在"}}]}],
      relations:[{id:"rel_cut",kind:{type:"transition",from:{type:"item",item_id:"itm_a"},
        to:{type:"item",item_id:"itm_b"},transition:{kind:{type:"dissolve"},
          duration:{timescale:600,value:240},alignment:"centered"}}}],
      relation_authorship:[{relation_id:"rel_cut",entity:{logical_path:
        ["executable-centered-dissolve","sequence","main","relation","scene-cut"]}}]}
  ' >"$spec"
  write_executable_project "$spec" "$dir/project/project.veac.json" \
    executable-centered-dissolve 640 360 30
  copy_executable_preview "$dir/project/project.veac.json" "$dir/project/project.preview.veac.json"
  jq '.project.render_configs[0].raster.width=480 |
      .project.render_configs[0].raster.height=270' \
    "$dir/project/project.preview.veac.json" >"$dir/project.preview.json"
  mv "$dir/project.preview.json" "$dir/project/project.preview.veac.json"
  jq -n '{tracks:[{clips:[{id:"itm_a",source_mapping:null},{id:"itm_b",source_mapping:null}],
    transitions:[{relation_id:"rel_cut",outgoing_clip_id:"itm_a",incoming_clip_id:"itm_b",
      kind:{type:"dissolve"},alignment:"centered",record_window:{start:{timescale:600,value:960},
        duration:{timescale:600,value:240}},cut_time:{timescale:600,value:1080},
      outgoing_range:{start:{timescale:600,value:960},duration:{timescale:600,value:240}},
      incoming_range:{start:{timescale:600,value:0},duration:{timescale:600,value:240}}}]}]}' \
    >"$plan_spec"
  write_executable_plan "$plan_spec" "$dir/plans/preview/out_preview.json" 4 480 270 discard null
  jq '.sequences[0].settings={width:640,height:360}' \
    "$dir/plans/preview/out_preview.json" >"$dir/plan.json"
  mv "$dir/plan.json" "$dir/plans/preview/out_preview.json"
}
