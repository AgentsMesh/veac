use veac_plan::canonical::*;
use veac_plan::ResolvedClipSource;

use super::support::{bindings, emit_video_command, fixture, resolved};

#[test]
fn linear_and_radial_gradients_emit_explicit_piecewise_rgba_geq() {
    for gradient in [linear(), radial()] {
        let mut plan = resolved(&fixture());
        clip(&mut plan).source = ResolvedClipSource::Generated {
            generator: Generator::Gradient { gradient },
        };
        let graph = graph(&plan);
        for marker in ["gradientcanvasv", "gradientv", "geq=r=", "max(0\\,min(1"] {
            assert!(graph.contains(marker), "missing {marker}: {graph}");
        }
    }
}

#[test]
fn every_vector_geometry_emits_fill_and_stroke_through_normal_visual_pipeline() {
    let geometries = vec![
        VectorGeometry::Rectangle { bounds: bounds() },
        VectorGeometry::Ellipse { bounds: bounds() },
        VectorGeometry::RoundedRectangle {
            bounds: bounds(),
            radius: 0.1,
        },
        VectorGeometry::Polygon { points: points() },
        VectorGeometry::Path {
            commands: vec![
                PathCommand::MoveTo { point: points()[0] },
                PathCommand::LineTo { point: points()[1] },
                PathCommand::LineTo { point: points()[2] },
                PathCommand::Close,
            ],
        },
    ];
    for geometry in geometries {
        let mut plan = resolved(&fixture());
        clip(&mut plan).source = ResolvedClipSource::Generated {
            generator: Generator::Shape {
                shape: VectorShape {
                    geometry,
                    fill: Some(Paint::Gradient { gradient: linear() }),
                    stroke: Some(VectorStroke {
                        paint: Paint::Solid { color: white() },
                        width_pixels: 3.0,
                    }),
                },
            },
        };
        let graph = graph(&plan);
        assert!(graph.contains("shapecanvasv"));
        assert!(graph.contains("shapev"));
        assert!(
            graph.contains("blend=all_expr='B+A*(65535-B)/65535'"),
            "missing source-over: {graph}"
        );
        assert!(
            graph.contains("mergeplanes=format=gbrap16le"),
            "shape must preserve high-precision alpha: {graph}"
        );
    }
    for shape in [
        VectorShape {
            geometry: VectorGeometry::Rectangle { bounds: bounds() },
            fill: Some(Paint::Solid { color: white() }),
            stroke: None,
        },
        VectorShape {
            geometry: VectorGeometry::Ellipse { bounds: bounds() },
            fill: None,
            stroke: Some(VectorStroke {
                paint: Paint::Solid { color: white() },
                width_pixels: 3.0,
            }),
        },
    ] {
        let mut plan = resolved(&fixture());
        clip(&mut plan).source = ResolvedClipSource::Generated {
            generator: Generator::Shape { shape },
        };
        let graph = graph(&plan);
        assert!(graph.contains("shapev"));
        assert!(graph.contains("if(gt(0\\,0)\\,0"));
    }
}

#[test]
fn malformed_generated_plan_fails_preflight_without_expression_panics() {
    let mut plan = resolved(&fixture());
    clip(&mut plan).source = ResolvedClipSource::Generated {
        generator: Generator::Gradient {
            gradient: Gradient::Linear {
                start: Vec2 { x: 0.0, y: 0.0 },
                end: Vec2 { x: 0.0, y: 0.0 },
                stops: vec![],
            },
        },
    };
    let error = emit_video_command(&plan, &bindings(&plan)).unwrap_err();
    assert!(error
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.code == "PLAN_GENERATOR_INVALID"));
}

fn clip(plan: &mut veac_plan::ResolvedRenderPlan) -> &mut veac_plan::ResolvedClip {
    &mut plan.sequences[0].tracks[0].clips[0]
}

fn graph(plan: &veac_plan::ResolvedRenderPlan) -> String {
    emit_video_command(plan, &bindings(plan))
        .unwrap()
        .filter_graph
        .unwrap()
}

fn linear() -> Gradient {
    Gradient::Linear {
        start: Vec2 { x: 0.0, y: 0.0 },
        end: Vec2 { x: 1.0, y: 1.0 },
        stops: stops(),
    }
}

fn radial() -> Gradient {
    Gradient::Radial {
        center: Vec2 { x: 0.5, y: 0.5 },
        radius: 0.6,
        stops: stops(),
    }
}

fn stops() -> Vec<GradientStop> {
    vec![
        GradientStop {
            offset: 0.0,
            color: white(),
        },
        GradientStop {
            offset: 0.4,
            color: Color {
                red: 255,
                green: 0,
                blue: 0,
                alpha: 180,
            },
        },
        GradientStop {
            offset: 1.0,
            color: Color {
                red: 0,
                green: 0,
                blue: 255,
                alpha: 0,
            },
        },
    ]
}

fn bounds() -> Rect {
    Rect {
        x: 0.1,
        y: 0.1,
        width: 0.8,
        height: 0.8,
    }
}

fn points() -> Vec<Vec2> {
    vec![
        Vec2 { x: 0.5, y: 0.1 },
        Vec2 { x: 0.9, y: 0.9 },
        Vec2 { x: 0.1, y: 0.9 },
    ]
}

fn white() -> Color {
    Color {
        red: 255,
        green: 255,
        blue: 255,
        alpha: 255,
    }
}
