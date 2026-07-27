# Sources

`source` is a closed sum type. A source variant owns only configuration meaningful to that variant.

Media and nested sequence:

```veac
source media resource camera-a;
source sequence sequence intro;
```

Generated sources:

```veac
source generated transparent;
source generated silence;
source generated solid { color #112233ff; }
source generated gradient linear {
  from 0% 0%; to 100% 100%;
  stop 0% #112233ff;
  stop 50% #6677aaff;
  stop 100% #ffeeccff;
}
source generated shape {
  geometry rounded-rectangle { bounds 0% 0% 100% 100%; radius 16px; }
  fill solid #ffffffff;
  stroke 2px solid #000000ff;
}
```

Multicam is a project entity plus an item-local switch program:

```veac
multicam interview {
  sync audio { reference angle host; }
  angle host { source resource host-video; source-offset 0s; }
  angle guest { source resource guest-video; source-offset 120ms; }
}

source multicam multicam interview {
  switch angle host { at 0s; duration 4s; }
  switch angle guest { at 4s; duration 3s; }
}
```

Switches must form a contiguous clip-local partition from zero through the complete item duration. Angles require video resources, and sync references must belong to the group.

Text and caption are structured sources rather than quoted declaration tails. Their shared style and layout primitives are described in [Text, captions, and audio](text-caption-audio.md).
