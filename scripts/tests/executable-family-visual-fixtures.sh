#!/usr/bin/env bash

write_local_image_fixture() {
  local root=$1 dir spec plan_spec
  dir=$(prepare_executable_fixture "$root" executable-local-image)
  spec="$dir/project.spec.json"; plan_spec="$dir/plan.spec.json"
  jq -n '
    {materials:[{id:"med_poster",kind:"image",source:{type:"file",uri:"assets/executable-local-image.png"},
      identity:{algorithm:"sha256",digest:"e8e9cbd056dda50d6ef53b8b73946e0d238d9dead67be2a2259113bbf9e043e5"},
      authorship:{logical_path:["executable-local-image","resource","poster"]}}],
      tracks:[{id:"trk_image",clips:[{id:"itm_poster",authorship:{logical_path:
        ["executable-local-image","sequence","main","layer","image","item","poster"]},
        source:{type:"media",material_id:"med_poster"},visual:{
          frame:{width:{value:56,unit:"pixels"},height:{value:30,unit:"pixels"},fit:"contain"},
          placement:{type:"anchor",anchor:"center",inset:{x:0,y:0}},transform:{anchor:{x:.25,y:.5},
            flip_horizontal:true,flip_vertical:false,position:{value:{y:{value:-1,unit:"pixels"}}},
            scale:{value:{x:.72,y:.72}},rotation_degrees:{value:8}}}}]}]}
  ' >"$spec"
  write_executable_project "$spec" "$dir/project/project.veac.json" executable-local-image 64 36 30
  copy_executable_preview "$dir/project/project.veac.json" "$dir/project/project.preview.veac.json"
  jq -n '{inputs:[{id:"pin_poster",material_id:"med_poster",kind:{type:"media",material_kind:"image"},
    canonical_uri:"assets/executable-local-image.png",observed_identity:{digest:
      "e8e9cbd056dda50d6ef53b8b73946e0d238d9dead67be2a2259113bbf9e043e5"},
    video:{info:{width:64,height:36,pixel_format:"monob"}}}],tracks:[{clips:[{id:"itm_poster",
      source:{type:"media",input_id:"pin_poster",video_stream:{global_index:0,type_index:0},audio_stream:null},
      visual:{frame:{fit:"contain"},transform:{flip_horizontal:true}}}]}]}' >"$plan_spec"
  write_executable_plan "$plan_spec" "$dir/plans/preview/out_preview.json" 3 64 36 discard null
}

write_text_family_fixture() {
  local root=$1 dir spec plan_spec
  dir=$(prepare_executable_fixture "$root" executable-text-family)
  spec="$dir/project.spec.json"; plan_spec="$dir/plan.spec.json"
  jq -n '
    def range($s): {start:{timescale:600,value:$s},duration:{timescale:600,value:1200}};
    def animation($key;$granularity): {granularity:$granularity,
      transform:{position_offset:{keyframes:[{id:("kf_"+$key+"_p0")},{id:("kf_"+$key+"_p1")}]}},
      reveal:{keyframes:[range(0;8)|{id:("kf_"+$key+"_r"+tostring)}]}};
    def layout($writing;$orientation): {writing_mode:$writing,orientation:$orientation};
    def clip($track;$key;$id;$start;$type;$text;$granularity;$writing;$orientation):
      {id:$id,authorship:{logical_path:["executable-text-family","sequence","main","layer",$track,"item",$key]},
       record_range:range($start),source:{type:$type,text:$text,style:{font:{material_id:"med_font"},
        fallback_fonts:[{type:"family",family:"PingFang SC"},{type:"family",family:"Noto Sans CJK SC"}],
        layout:layout($writing;$orientation),animation:animation($key;$granularity),spans:[],path:null}}};
    clip("text";"word";"itm_word";0;"text";"逐词动画：代码化视频";"word";"horizontal-tb";"mixed") as $word |
    clip("captions";"grapheme";"itm_grapheme";1200;"caption";"逐字动画：中文可观察";"grapheme";"vertical-rl";"upright") as $grapheme |
    clip("captions";"line";"itm_line";2400;"caption";"逐行动画：布局与装饰";"line";"vertical-lr";"mixed") as $line |
    clip("text";"whole";"itm_whole";3600;"text";"整体动画：路径文字";"whole";"horizontal-tb";"sideways") as $whole |
    ($word | .source.style.spans=[{start:0,end:4,font:{type:"family",family:"sans-serif"}}]) as $word |
    ($whole | .source.style.path={points:[{},{},{}],start_offset:{value:286,unit:"pixels"}}) as $whole |
    {materials:[{id:"med_font",kind:"font",source:{uri:"assets/veac-example-zh.ttf"},
      identity:{digest:"64afada094cf447d71fad4ee726799c4fa74917f7d9a1113ed7ea3c4234ff774"},authorship:{logical_path:
      ["executable-text-family","resource","example-font"]}}],
      tracks:[{id:"trk_text",clips:[$word,$whole]},{id:"trk_captions",clips:[$grapheme,$line]}]}
  ' >"$spec"
  write_executable_project "$spec" "$dir/project/project.veac.json" executable-text-family 640 360 30
  jq '
    .project.sequences[0].settings.frame_rate.numerator=12 |
    .project.materials += [{id:"med_fallback",kind:"font"},{id:"med_span",kind:"font"}] |
    (.project.sequences[].tracks[].clips[]|select(.source.type=="text" or .source.type=="caption")|
      .source.style.fallback_fonts)=[{type:"material",material_id:"med_fallback"},
        {type:"material",material_id:"med_fallback"}] |
    (.project.sequences[].tracks[].clips[]|select(.id=="itm_word")|
      .source.style.spans[0].font)={type:"material",material_id:"med_span"}
  ' "$dir/project/project.veac.json" >"$dir/project/project.preview.veac.json"
  jq -n '
    def style($granularity;$writing): {animation:{granularity:$granularity},
      layout:{writing_mode:$writing},font:{requested:{material_id:"med_font"}},
      fallback_fonts:[{},{}],spans:[],path:null};
    def clip($id;$text;$granularity;$writing): {id:$id,source:{content:{text:$text,
      presentation:{style:style($granularity;$writing)}}}};
    clip("itm_word";"逐词动画：代码化视频";"word";"horizontal-tb") as $word |
    clip("itm_grapheme";"逐字动画：中文可观察";"grapheme";"vertical-rl") as $grapheme |
    clip("itm_line";"逐行动画：布局与装饰";"line";"vertical-lr") as $line |
    clip("itm_whole";"整体动画：路径文字";"whole";"horizontal-tb") as $whole |
    ($word | .source.content.presentation.style.spans=[{}]) as $word |
    ($whole | .source.content.presentation.style.path={points:[{},{},{}]}) as $whole |
    {inputs:[{material_id:"med_font",canonical_uri:"assets/veac-example-zh.ttf",
      observed_identity:{digest:"64afada094cf447d71fad4ee726799c4fa74917f7d9a1113ed7ea3c4234ff774"}}],
      tracks:[{clips:[$word,$grapheme,$line,$whole]}]}
  ' >"$plan_spec"
  write_executable_plan "$plan_spec" "$dir/plans/preview/out_preview.json" 8 480 270 burn_in null
}

write_executable_family_projects() {
  local root=$1
  write_programming_fixture "$root"
  write_audio_caption_fixture "$root"
  write_dissolve_fixture "$root"
  write_local_image_fixture "$root"
  write_text_family_fixture "$root"
}
