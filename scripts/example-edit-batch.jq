def one($label):
  if length == 1 then .[0] else error("expected one " + $label) end;

def clips: .project.sequences[].tracks[].clips[];
def clip($key):
  [clips | select(.authorship.logical_path[-1] == $key)] | one("clip " + $key);
def owner($root; $id):
  [$root.project.sequences[].tracks[] | select(any(.clips[]; .id == $id))]
  | one("owner track for " + $id);
def right_id($clip):
  "itm_workflow_" + ($clip.authorship.logical_path[-1] | gsub("[^a-z0-9]+"; "_")) + "_right";

. as $root |
(clip("first")) as $first |
(clip("second")) as $second |
(clip("trimmable")) as $trimmable |
(clip("linked-video")) as $video |
(clip("linked-audio")) as $audio |
(clip("removable")) as $removable |
([.project.relations[] | select(.kind.type == "group")] | one("group relation")) as $group |
([.project.relations[] | select(.kind.type == "av_link")] | one("AV link")) as $link |
if (($group.kind.members | map(.item_id) | sort) != ([$first.id, $second.id] | sort)) then
  error("group members do not match authored scene items")
elif $link.kind.video.item_id != $video.id or
    ($link.kind.audio | map(.item_id)) != [$audio.id] then
  error("AV link does not match authored linked items")
else
  [$first, $second, $trimmable, $video, $audio, $removable] as $items |
  ([$items[] | owner($root; .id)] | unique_by(.id)) as $tracks |
  {
    operation_id: "op_example_atomic_edit",
    base_revision: .project.revision,
    atomic: true,
    preconditions:
      ([$items[] | {type:"clip_exists", clip_id:.id}] +
       [$tracks[] | {type:"track_unlocked", track_id:.id}]),
    operations: [
      {type:"move_clip", clip_id:$first.id,
       record_start:{value:300, timescale:600}},
      {type:"trim_clip", clip_id:$trimmable.id, edge:"out",
       delta:{value:-300, timescale:600}, ripple:false},
      {type:"split_linked", link_relation_id:$link.id,
       offset:{value:1200, timescale:600},
       fragments:[
         {source_clip_id:$video.id, right_clip_id:right_id($video), relation_fragments:[]},
         {source_clip_id:$audio.id, right_clip_id:right_id($audio), relation_fragments:[]}
       ], right_link_relation_id:"rel_workflow_linked_av_right"},
      {type:"remove_clip", clip_id:$removable.id}
    ]
  }
end
