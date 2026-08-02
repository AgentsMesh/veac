# Agent Reuse Guide

Put shared declarations in a `module`, export only its public surface, and import it through a
stable alias. Prefer typed constants and presets over repeating values or blocks. Use a
`component sequence` when parameters and source slots describe one reusable timeline structure:

```veac,fragment
import "./brand.veac" as brand;
const time duration = brand.card_duration + 500ms;
instance sequence opener from brand.title_card {
    bind title "章节标题";
    fill backdrop { source generated solid { color #18384bff; } }
}
```

Expressions are pure and exact. A component-local `@title` receives a reserved, length-framed
generated ID. Edit the `@title` declaration; never write its generated ID directly.

Compose reusable timelines with the same instance primitive inside a component, using `@` for the
owned child sequence:

```veac,fragment
component sequence chapter {
    param time duration;
    slot visual background;
    instance sequence @card from title_card {
        bind duration duration;
        fill backdrop { source slot background; }
    }
    body {
        layer visual @content { item @card-item {
            source sequence sequence @card;
            record { at 0s; duration ${duration}; }
        } }
    }
}
```

Pass data and slots explicitly. A child cannot capture the outer entry caller. Canonical IR contains
only the expanded child sequence plus the parent's ordinary sequence reference; components,
instances, and lexical scopes remain compile-time mechanisms.
