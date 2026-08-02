use crate::authoring::*;

use super::writer::Writer;

pub(super) fn relation(out: &mut Writer, value: &RelationDecl) {
    let kind = match value.kind {
        RelationKind::Transition(_) => "transition",
        RelationKind::Matte(_) => "matte",
        RelationKind::Sidechain(_) => "sidechain",
        RelationKind::Group(_) => "group",
        RelationKind::AvLink(_) => "av-link",
    };
    out.block(
        format!("relation {kind} {}", value.id.value),
        |out| match &value.kind {
            RelationKind::Transition(value) => transition(out, value),
            RelationKind::Matte(value) => matte(out, value),
            RelationKind::Sidechain(value) => sidechain(out, value),
            RelationKind::Group(value) => group(out, value),
            RelationKind::AvLink(value) => av_link(out, value),
        },
    );
}

fn transition(out: &mut Writer, value: &TransitionRelation) {
    out.block("endpoints", |out| {
        item(out, "from", &value.endpoints.from);
        item(out, "to", &value.endpoints.to);
    });
    out.block("timing", |out| {
        out.line(format!("duration {};", value.timing.duration.raw));
        let alignment = match value.timing.alignment.value {
            TransitionAlignment::BeforeCut => "before-cut",
            TransitionAlignment::Centered => "centered",
            TransitionAlignment::AfterCut => "after-cut",
        };
        out.line(format!("alignment {alignment};"));
    });
    out.block("style", |out| transition_style(out, &value.style));
}

fn transition_style(out: &mut Writer, value: &TransitionStyle) {
    match value {
        TransitionStyle::Dissolve => out.line("dissolve;"),
        TransitionStyle::Fade { color } => out.block("fade", |out| {
            let color = match color.value {
                FadeColor::Transparent => "transparent",
                FadeColor::Black => "black",
                FadeColor::White => "white",
            };
            out.line(format!("color {color};"));
        }),
        TransitionStyle::Wipe {
            direction,
            angle,
            softness,
        } => out.block("wipe", |out| {
            out.line(format!("direction {};", cardinal(direction.value)));
            out.line(format!("angle {};", angle.raw));
            out.line(format!("softness {};", softness.raw));
        }),
        TransitionStyle::Slide { direction, amount } => out.block("slide", |out| {
            out.line(format!("direction {};", cardinal(direction.value)));
            out.line(format!("amount {};", amount.raw));
        }),
        TransitionStyle::Zoom { direction, amount } => out.block("zoom", |out| {
            let direction = match direction.value {
                ZoomDirection::In => "in",
                ZoomDirection::Out => "out",
            };
            out.line(format!("direction {direction};"));
            out.line(format!("amount {};", amount.raw));
        }),
        TransitionStyle::Circle {
            direction,
            softness,
        } => out.block("circle", |out| {
            let direction = match direction.value {
                CircleDirection::Open => "open",
                CircleDirection::Close => "close",
            };
            out.line(format!("direction {direction};"));
            out.line(format!("softness {};", softness.raw));
        }),
        TransitionStyle::Pixelize { amount } => out.block("pixelize", |out| {
            out.line(format!("amount {};", amount.raw));
        }),
    }
}

fn matte(out: &mut Writer, value: &MatteRelation) {
    out.block("endpoints", |out| {
        item(out, "producer", &value.endpoints.producer);
        item(out, "consumer", &value.endpoints.consumer);
    });
    out.block("style", |out| {
        let mode = match value.style.mode.value {
            MatteMode::Alpha => "alpha",
            MatteMode::Luma => "luma",
        };
        out.block(mode, |out| {
            out.line(format!("invert {};", value.style.invert.value));
        });
    });
}

fn sidechain(out: &mut Writer, value: &SidechainRelation) {
    out.block("endpoints", |out| {
        let (kind, id) = match &value.endpoints.key {
            SignalEndpoint::Track { id } => ("track", &id.value),
            SignalEndpoint::Bus { id } => ("bus", &id.value),
        };
        out.line(format!("key {kind} {id};"));
        item(out, "target", &value.endpoints.target);
    });
    out.block("dynamics", |out| {
        out.line(format!("threshold {};", value.dynamics.threshold.raw));
        out.line(format!("ratio {};", value.dynamics.ratio.raw));
        out.line(format!("attack {};", value.dynamics.attack.raw));
        out.line(format!("release {};", value.dynamics.release.raw));
    });
    if let Some(active) = &value.timing.active {
        out.block("timing", |out| {
            out.block("active", |out| {
                out.line(format!("at {};", active.at.raw));
                out.line(format!("duration {};", active.duration.raw));
            });
        });
    }
}

fn group(out: &mut Writer, value: &GroupRelation) {
    out.block("members", |out| {
        for member in &value.members {
            out.line(format!("item {};", member.id.value));
        }
    });
}

fn av_link(out: &mut Writer, value: &AvLinkRelation) {
    out.block("endpoints", |out| {
        item(out, "video", &value.video);
        out.block("audio", |out| {
            for member in &value.audio {
                out.line(format!("item {};", member.id.value));
            }
        });
    });
}

fn item(out: &mut Writer, role: &str, endpoint: &ItemEndpoint) {
    out.line(format!("{role} item {};", endpoint.id.value));
}

fn cardinal(value: CardinalDirection) -> &'static str {
    match value {
        CardinalDirection::Left => "left",
        CardinalDirection::Right => "right",
        CardinalDirection::Up => "up",
        CardinalDirection::Down => "down",
    }
}
