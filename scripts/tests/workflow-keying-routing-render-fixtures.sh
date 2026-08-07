#!/usr/bin/env bash

write_keying_contract_fixtures() {
  local dir=$1; mkdir -p "$dir"
  jq -n '
    def rr($s;$d): {start:{value:$s,timescale:600},duration:{value:$d,timescale:600}};
    def auth($key): {logical_path:["fixture","item",$key]};
    def map: {frame_synthesis:"nearest",out_of_range:"strict",time_map:{direction:"forward",
      rate:{denominator:1,numerator:1},repeat:1,source_start:{value:0,timescale:600},type:"linear"}};
    def item($id;$key;$s;$d;$source;$effects): {id:$id,authorship:auth($key),
      record_range:rr($s;$d),source:$source,source_mapping:(if $source.type=="media" then map else null end),
      effects:$effects};
    def effect($id;$value): {id:$id,effect:$value,enable_range:null,enabled:true};
    {project:{materials:[
      {id:"med_plate",authorship:auth("plate"),kind:"image"},
      {id:"med_shaky",authorship:auth("shaky"),kind:"video"}],
      sequences:[{id:"seq_main",authorship:{type:"veac",entity:auth("main")},tracks:[{clips:[
        item("itm_chroma";"chroma";0;600;{type:"media",material_id:"med_plate"};[
          effect("fx_chroma";{type:"video_chroma_key",color:{alpha:255,blue:100,green:194,red:45},
            similarity:{type:"constant",value:0.18},blend:{type:"constant",value:0.08}}),
          effect("fx_spill";{type:"video_chroma_spill",color:{alpha:255,blue:100,green:194,red:45},
            range:{type:"constant",value:0.2},amount:{type:"constant",value:1}})]),
        item("itm_luma";"luma";600;600;{type:"media",material_id:"med_plate"};[
          effect("fx_luma";{type:"video_luma_key",threshold:{type:"constant",value:0.12},
            tolerance:{type:"constant",value:0.08},softness:{type:"constant",value:0.08},invert:false})]),
        item("itm_stable";"stable";1200;1200;{type:"media",material_id:"med_shaky"};[
          effect("fx_stable";{type:"video_stabilize",enabled:true})]),
        item("itm_key_label";"key-label";0;1200;
          {type:"text",text:"色度键、溢色抑制与亮度键"};[]),
        item("itm_stable_label";"stable-label";1200;1200;
          {type:"text",text:"确定性抖动素材：防抖后"};[])
      ]}]}]}}
  ' >"$dir/keying-author.json"
  jq '
    def rr($d): {start:{value:0,timescale:600},duration:{value:$d,timescale:600}};
    {inputs:[
      {id:"pin_plate",material_id:"med_plate",canonical_uri:"assets/effects-plate.png",
       kind:{material_kind:"image",type:"media"},audio:null,video:{selection:{global_index:0,type_index:0}}},
      {id:"pin_shaky",material_id:"med_shaky",canonical_uri:"assets/shaky.mp4",
       kind:{material_kind:"video",type:"media"},audio:null,
       video:{selection:{global_index:0,type_index:0},duration:{value:2,timescale:1}}}],
     sequences:.project.sequences}
    | (.sequences[].tracks[].clips[] | select(.source.type=="media")) |=
        (.source={type:"media",input_id:(if .id=="itm_stable" then "pin_shaky" else "pin_plate" end),
          audio_stream:null,video_stream:{global_index:0,type_index:0}}
         | .source_mapping={frame_synthesis:"nearest",out_of_range:"strict",time_map:{direction:"forward",
            rate:{denominator:1,numerator:1},repeat:1,type:"linear",
            source_range_per_repeat:rr(.record_range.duration.value)}})
    | (.sequences[].tracks[].clips[] | select(.source.type=="text")) |=
        (.source={type:"text",content:{text:.source.text}})
    | (.sequences[].tracks[].clips[] | select(.id=="itm_chroma" or .id=="itm_luma")) |=
        (.effects |= map({id,effect,active_range:rr(600)}))
    | (.sequences[].tracks[].clips[] | select(.id=="itm_stable")) |=
        (.effects |= map({id,effect,active_range:rr(1200)}))
  ' "$dir/keying-author.json" >"$dir/keying-plan.json"
}

write_routing_contract_fixtures() {
  local dir=$1; mkdir -p "$dir"
  jq -n '
    def rr: {start:{value:0,timescale:600},duration:{value:1200,timescale:600}};
    def auth($key): {logical_path:["fixture","item",$key]};
    def map: {frame_synthesis:"nearest",out_of_range:"strict",time_map:{direction:"forward",
      rate:{denominator:1,numerator:1},repeat:1,source_start:{value:0,timescale:600},type:"linear"}};
    def audio($gain;$pan): {crossfade:{curve:"equal_power",fade_in:{timescale:600,value:90},
      fade_out:{timescale:600,value:90}},gain:{type:"constant",value:$gain},muted:false,
      normalize:false,pan:{type:"constant",value:$pan},pitch_policy:"preserve",processors:[]};
    def item($id;$key;$source;$audio): {id:$id,authorship:auth($key),record_range:rr,
      source:$source,source_mapping:(if $source.type=="media" then map else null end),audio:$audio};
    {project:{materials:[{id:"med_tone",authorship:auth("tone"),kind:"audio"}],sequences:[
      {id:"seq_main",authorship:{type:"veac",entity:auth("main")},tracks:[
        {id:"trk_music",routing:{bus_id:"bus_music-bus",type:"audio_bus"},clips:[
          item("itm_music";"music";{type:"media",material_id:"med_tone"};audio(0.75;-0.25))]},
        {id:"trk_key",routing:{type:"default"},clips:[
          item("itm_key";"key";{type:"media",material_id:"med_tone"};audio(0.5;0.25))]},
        {id:"trk_label",routing:{type:"default"},clips:[item("itm_label";"explanation";
          {type:"text",text:"音频总线路由与侧链压低进入解析计划"};null)]}
      ]}],relations:[{id:"rel_sidechain",sequence_id:"seq_main",kind:{type:"sidechain",
        key:{type:"track",track_id:"trk_key"},target:{type:"item",item_id:"itm_music"},
        parameters:{active_range:null,attack_ms:10,ratio:4,release_ms:120,threshold_db:-24}}}]}}
  ' >"$dir/routing-author.json"
  jq '
    def rr: {start:{value:0,timescale:600},duration:{value:1200,timescale:600}};
    {inputs:[{id:"pin_tone",material_id:"med_tone",canonical_uri:"assets/tone.wav",
      kind:{material_kind:"audio",type:"media"},audio:{codec:"pcm_s16le",
        info:{channel_layout:"unknown",channels:1,sample_rate:48000},
        selection:{global_index:0,type_index:0}},video:null}],sequences:.project.sequences}
    | (.sequences[].tracks[] | select(.id=="trk_music")).routing=
        {audio:{bus_id:"bus_music-bus",type:"bus"},visual:null}
    | (.sequences[].tracks[] | select(.id=="trk_key")).routing=
        {audio:{type:"main_mix"},visual:null}
    | (.sequences[].tracks[].clips[] | select(.source.type=="media")) |=
        (.source={type:"media",input_id:"pin_tone",audio_stream:{global_index:0,type_index:0},
          video_stream:null}|.source_mapping={frame_synthesis:"nearest",out_of_range:"strict",
            time_map:{direction:"forward",rate:{denominator:1,numerator:1},repeat:1,
              source_range_per_repeat:rr,type:"linear"}})
    | (.sequences[].tracks[].clips[] | select(.id=="itm_music")).audio.sidechain=
        {active_range:null,attack_ms:10,ratio:4,relation_id:"rel_sidechain",release_ms:120,
         source:{track_id:"trk_key",type:"track"},threshold_db:-24}
    | (.sequences[].tracks[].clips[] | select(.id=="itm_key")).audio.sidechain=null
    | (.sequences[].tracks[].clips[] | select(.source.type=="text")).source=
        {type:"text",content:{text:"音频总线路由与侧链压低进入解析计划"}}
  ' "$dir/routing-author.json" >"$dir/routing-plan.json"
}
