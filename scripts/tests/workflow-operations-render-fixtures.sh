#!/usr/bin/env bash

write_edit_contract_fixtures() {
  local dir=$1; mkdir -p "$dir"
  jq -n '
    def rr($s;$d): {start:{value:$s,timescale:600},duration:{value:$d,timescale:600}};
    def auth($key): {logical_path:["fixture","item",$key]};
    def item($id;$key;$s;$d;$source): {id:$id,authorship:auth($key),
      record_range:rr($s;$d),source:$source,source_mapping:null};
    {project:{sequences:[{id:"seq_main",authorship:{type:"veac",entity:auth("main")},tracks:[
      {id:"trk_scenes",clips:[
        item("itm_first";"first";0;1200;{type:"generated",generator:{type:"solid",
          color:{alpha:255,blue:235,green:99,red:37}}}),
        item("itm_second";"second";1200;1200;{type:"generated",generator:{type:"solid",
          color:{alpha:255,blue:119,green:39,red:219}}})]},
      {id:"trk_labels",clips:[item("itm_text";"explanation";0;2400;
        {type:"text",text:"编辑操作：分组移动、独立裁剪、音视频同时分割与删除"})]},
      {id:"trk_video",clips:[item("itm_linked_video";"linked-video";0;2400;
        {type:"generated",generator:{type:"transparent"}})]},
      {id:"trk_audio",clips:[item("itm_linked_audio";"linked-audio";0;2400;
        {type:"generated",generator:{type:"silence"}})]},
      {id:"trk_trim",clips:[item("itm_trim";"trimmable";0;2400;
        {type:"generated",generator:{type:"transparent"}})]},
      {id:"trk_remove",clips:[item("itm_remove";"removable";0;2400;
        {type:"generated",generator:{type:"transparent"}})]}
    ]}],relations:[
      {id:"rel_group",sequence_id:"seq_main",kind:{type:"group",members:[{type:"item",item_id:"itm_first"},
        {type:"item",item_id:"itm_second"}]}},
      {id:"rel_av",sequence_id:"seq_main",kind:{type:"av_link",video:{type:"item",item_id:"itm_linked_video"},
        audio:[{type:"item",item_id:"itm_linked_audio"}]}}
    ]}}
  ' >"$dir/edit-author.json"
  jq '
    {sequences:.project.sequences}
    | .header={schema:"https://veac.dev/schemas/render-plan",schema_version:6,
        resolver:{capability_profile:"backend-neutral-v1",effect_registry_version:"veac-ir-effects-v2",
          resolver_version:"veac-plan-resolver-v6",stream_selection_policy:"none"},source:{timebase:600}}
    | .entry_sequence_id="seq_main" | .temporal={opset_version:1}
    | .output={sequence_id:"seq_main",raster:{captions:"discard",
        frame_rate:{denominator:1,numerator:12},height:270,width:480},deliverables:[{
        target:{name:"preview.mp4",type:"file"},kind:{type:"video",settings:{audio:null,
          container:"mp4",hardware:{type:"software"},optimize_for_streaming:true,pass_mode:"single",
          video:{alpha:"opaque",b_frames:null,codec:"h264",color_space:null,gop_size:null,level:null,
            pixel_format:"yuv420p",profile:"h264_high",rate_control:{type:"crf",value:20}}}}}]}
    | .sequences[0].duration={value:2400,timescale:600}
    | .sequences[0].settings={frame_rate:{denominator:1,numerator:12},height:360,
        sample_rate:48000,width:640}
    | (.sequences[].tracks[].clips) |= map(select(.id=="itm_first" or .id=="itm_second" or
        .id=="itm_text"))
    | (.sequences[].tracks[].clips[] | select(.id=="itm_text")).source=
        {type:"text",content:{text:"编辑操作：分组移动、独立裁剪、音视频同时分割与删除"}}
    | (.sequences[].tracks[] | select(.id=="trk_audio")).kind="audio"
    | (.sequences[].tracks[] | select(.id!="trk_audio")).kind="visual"
  ' "$dir/edit-author.json" >"$dir/edit-plan.json"
}

write_probe_contract_fixtures() {
  local dir=$1; mkdir -p "$dir"
  jq -n '
    def rr: {start:{value:0,timescale:600},duration:{value:1800,timescale:600}};
    def auth($key): {logical_path:["fixture","item",$key]};
    {project:{materials:[{id:"med_source",authorship:auth("source"),kind:"video",
      source:{type:"file",uri:"assets/source.mp4"},stream_intent:{audio:{type:"disabled"},
        video:{global_index:0,type:"global_index"}}}],sequences:[{id:"seq_main",
      authorship:{type:"veac",entity:auth("main")},tracks:[{clips:[
        {id:"itm_stream",authorship:auth("selected-stream"),record_range:rr,
          source:{type:"media",material_id:"med_source"}},
        {id:"itm_text",authorship:auth("explanation"),record_range:rr,
          source:{type:"text",text:"探测快照后显式选择全局零号视频流，并禁用音频流"}}
      ]}]}]}}
  ' >"$dir/probe-author.json"
  jq '
    {inputs:[{id:"pin_source",material_id:"med_source",canonical_uri:"assets/source.mp4",audio:null,
      video:{selection:{global_index:0,type_index:0}}}],sequences:.project.sequences}
    | (.sequences[].tracks[].clips[] | select(.id=="itm_stream")).source=
        {type:"media",input_id:"pin_source",audio_stream:null,
          video_stream:{global_index:0,type_index:0}}
    | (.sequences[].tracks[].clips[] | select(.id=="itm_text")).source=
        {type:"text",content:{text:"探测快照后显式选择全局零号视频流，并禁用音频流"}}
  ' "$dir/probe-author.json" >"$dir/probe-plan.json"
}
