def delivery_source: "examples/delivery-formats/main.veac";

def delivery_capabilities: ["P1-30", "P1-32"];

def delivery_artifacts: [
  "master",
  "frames",
  "transcript",
  "master-audio",
  "video-waveform",
  "podcast",
  "loop-preview",
  "cover",
  "stream"
];

def delivery_mechanisms: [
  "delivery.container.mp4",
  "delivery.video-codec.h264",
  "delivery.audio-codec.aac",
  "delivery.image-sequence",
  "delivery.image-format.png",
  "delivery.sidecar.captions",
  "delivery.caption-format.web-vtt",
  "delivery.audio-only",
  "delivery.audio-stem-format.wav",
  "delivery.audio-codec.pcm-s24le",
  "delivery.scope.waveform",
  "delivery.audio-codec.mp3",
  "delivery.gif",
  "delivery.single-frame",
  "delivery.multiresolution-hls"
];

(.targets | map(select(.id == "delivery-formats"))) as $targets |
(.examples | map(select(.id == "delivery-formats"))) as $examples |
($targets | length == 1) and
($examples | length == 1) and
($examples[0].source == delivery_source and
  ($examples[0].checks | length == (delivery_artifacts | length))) and
($targets[0] as $target |
  $target.kind == "workflow" and
  $target.example == delivery_source and
  $target.capability_ids == delivery_capabilities and
  ([$target.expected_artifacts[] |
    select(.kind == "authoring_delivery" and .id == "master") |
    .artifact_ids] == [delivery_artifacts])) and
(["P1-30", "P1-32"] | all(. as $id |
  any($capabilities[0].capabilities[];
    .id == $id and .example == delivery_source and
    (has("not_applicable") | not)))) and
([delivery_mechanisms[]] as $ids |
  ([$mechanisms[0][] | select(.id as $id | $ids | index($id))] | length) ==
    ($ids | length) and
  all($mechanisms[0][];
    (.id as $id | $ids | index($id)) == null or
    (.coverage == "workflow_evidence" and .evidence == delivery_source)))
