def one($label):
  if length == 1 then .[0] else error("expected one " + $label) end;
def clips($envelope): $envelope.project.sequences[].tracks[].clips[];
def clip_key($envelope; $key):
  [clips($envelope) | select(.authorship.logical_path[-1] == $key)]
  | one("clip " + $key);
def clip_id($envelope; $id):
  [clips($envelope) | select(.id == $id)] | one("clip " + $id);
def owner($envelope; $id):
  [$envelope.project.sequences[].tracks[] | select(any(.clips[]; .id == $id))]
  | one("owner track for " + $id);
def missing_clip($envelope; $id):
  [clips($envelope) | select(.id == $id)] | length == 0;
def operation($batch; $type):
  [$batch.operations[] | select(.type == $type)] | one($type + " operation");
def has_change($outcome; $type; $id):
  any($outcome.changed_objects[]; .type == $type and .id == $id);
def right_id($clip):
  "itm_workflow_" + ($clip.authorship.logical_path[-1] | gsub("[^a-z0-9]+"; "_")) + "_right";

($authoring[0]) as $before |
($batch[0]) as $request |
($replay[0]) as $again |
. as $applied |
(clip_key($before; "first")) as $first |
(clip_key($before; "second")) as $second |
(clip_key($before; "trimmable")) as $trimmable |
(clip_key($before; "linked-video")) as $video |
(clip_key($before; "linked-audio")) as $audio |
(clip_key($before; "removable")) as $removable |
(operation($request; "move_clip")) as $move |
(operation($request; "trim_clip")) as $trim |
(operation($request; "split_linked")) as $split |
(operation($request; "remove_clip")) as $remove |
([ $before.project.relations[] | select(.kind.type == "group") ]
  | one("group relation")) as $group |
([ $before.project.relations[] | select(.kind.type == "av_link") ]
  | one("AV link")) as $link |
(clip_id($applied.project; $first.id)) as $moved_first |
(clip_id($applied.project; $second.id)) as $moved_second |
(clip_id($applied.project; $trimmable.id)) as $trimmed_item |
(clip_id($applied.project; $video.id)) as $left_video |
(clip_id($applied.project; $audio.id)) as $left_audio |
(clip_id($applied.project; right_id($video))) as $right_video |
(clip_id($applied.project; right_id($audio))) as $right_audio |
($move.record_start.value - $first.record_range.start.value) as $move_delta |
($split.offset.value) as $split_value |
([$first, $second, $trimmable, $video, $audio, $removable]) as $affected |
([$affected[] | owner($before; .id)] | unique_by(.id)) as $affected_tracks |

$request.atomic == true and
$request.base_revision == $before.project.revision and
$before.project.timebase == 600 and
([$request.operations[].type] == ["move_clip", "trim_clip", "split_linked", "remove_clip"]) and
$move.clip_id == $first.id and
$trim.clip_id == $trimmable.id and $trim.edge == "out" and $trim.ripple == false and
$split.link_relation_id == $link.id and
$remove.clip_id == $removable.id and
([$first.id, $second.id, $trimmable.id, $video.id, $audio.id, $removable.id] |
  length == (unique | length)) and
($request.preconditions | sort_by(.type, .clip_id // .track_id)) ==
  (([$affected[] | {type:"clip_exists", clip_id:.id}] +
    [$affected_tracks[] | {type:"track_unlocked", track_id:.id}])
   | sort_by(.type, .clip_id // .track_id)) and
($group.kind.members | map(.item_id) | sort) == ([$first.id, $second.id] | sort) and
$link.kind.video.item_id == $video.id and
($link.kind.audio | map(.item_id)) == [$audio.id] and
($split.fragments | map([.source_clip_id, .right_clip_id])) == [
  [$video.id, right_id($video)], [$audio.id, right_id($audio)]] and
$split.right_link_relation_id == "rel_workflow_linked_av_right" and
$applied.status == "applied" and
$applied.new_revision == ($before.project.revision + 1) and
$applied.project.project.revision == $applied.new_revision and
([$applied.normalized_operations[].type] == [$request.operations[].type]) and
$moved_first.record_range.start.value == $move.record_start.value and
$moved_second.record_range.start.value == ($second.record_range.start.value + $move_delta) and
$trimmed_item.record_range.duration.value ==
  ($trimmable.record_range.duration.value + $trim.delta.value) and
$left_video.record_range.duration.value == $split_value and
$left_audio.record_range.duration.value == $split_value and
$right_video.record_range.start.value == ($video.record_range.start.value + $split_value) and
$right_audio.record_range.start.value == ($audio.record_range.start.value + $split_value) and
$right_video.record_range.duration.value == ($video.record_range.duration.value - $split_value) and
$right_audio.record_range.duration.value == ($audio.record_range.duration.value - $split_value) and
missing_clip($applied.project; $removable.id) and
any($applied.project.project.relations[];
  .id == $split.right_link_relation_id and .kind.type == "av_link" and
  .kind.video.item_id == $right_video.id and
  (.kind.audio | map(.item_id)) == [$right_audio.id]) and
any($applied.project.project.relations[];
  .id == $group.id and .kind == $group.kind) and
any($applied.project.project.relations[];
  .id == $link.id and .kind.type == "av_link" and
  .kind.video.item_id == $left_video.id and
  (.kind.audio | map(.item_id)) == [$left_audio.id]) and
([$first.id, $second.id, $trimmable.id, $video.id, $audio.id,
   $removable.id, $right_video.id, $right_audio.id] |
  all(.[]; has_change($applied; "item"; .))) and
any($applied.project.project.applied_operations[]; .id == $request.operation_id) and
$again.status == "no_change" and $again.operation_recorded == true and
$again.current_revision == $applied.new_revision and
$again.project == $applied.project
