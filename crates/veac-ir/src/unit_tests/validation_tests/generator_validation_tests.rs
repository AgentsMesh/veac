use super::*;

#[test]
fn every_gradient_and_vector_geometry_is_validated() {
    for gradient in [linear(), radial()] {
        validate(&generated(Generator::Gradient { gradient })).unwrap();
    }
    for geometry in [
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
    ] {
        validate(&generated(Generator::Shape {
            shape: VectorShape {
                geometry,
                fill: Some(Paint::Gradient { gradient: linear() }),
                stroke: Some(VectorStroke {
                    paint: Paint::Solid { color: white() },
                    width_pixels: 2.0,
                }),
            },
        }))
        .unwrap();
    }
}

#[test]
fn malformed_shape_styles_geometry_paths_and_gradients_fail_closed() {
    let invalid = [
        Generator::Shape {
            shape: VectorShape {
                geometry: VectorGeometry::Rectangle { bounds: bounds() },
                fill: None,
                stroke: None,
            },
        },
        shape(VectorGeometry::Rectangle {
            bounds: Rect {
                width: 2.0,
                ..bounds()
            },
        }),
        shape(VectorGeometry::RoundedRectangle {
            bounds: bounds(),
            radius: 0.9,
        }),
        shape(VectorGeometry::Polygon {
            points: vec![points()[0], points()[0], points()[1]],
        }),
        shape(VectorGeometry::Path {
            commands: vec![
                PathCommand::LineTo { point: points()[0] },
                PathCommand::LineTo { point: points()[1] },
                PathCommand::LineTo { point: points()[2] },
                PathCommand::Close,
            ],
        }),
        Generator::Gradient {
            gradient: Gradient::Radial {
                center: Vec2 { x: 2.0, y: 0.5 },
                radius: -1.0,
                stops: stops(),
            },
        },
    ];
    for generator in invalid {
        let codes = validation_codes(&generated(generator));
        assert!(codes.iter().any(|code| code.starts_with("GENERATOR_")));
    }
    let mut bad_stroke = match shape(VectorGeometry::Ellipse { bounds: bounds() }) {
        Generator::Shape { shape } => shape,
        _ => unreachable!(),
    };
    bad_stroke.stroke = Some(VectorStroke {
        paint: Paint::Solid { color: white() },
        width_pixels: -1.0,
    });
    assert_code(
        &validation_codes(&generated(Generator::Shape { shape: bad_stroke })),
        "GENERATOR_SHAPE_STYLE",
    );
}

fn generated(generator: Generator) -> ProjectEnvelope {
    let mut project = sample_project();
    let clip = &mut project.project.sequences[0].tracks[0].clips[0];
    clip.source_mapping = None;
    clip.audio = None;
    clip.source = ClipSource::Generated { generator };
    project
}

fn shape(geometry: VectorGeometry) -> Generator {
    Generator::Shape {
        shape: VectorShape {
            geometry,
            fill: Some(Paint::Solid { color: white() }),
            stroke: None,
        },
    }
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
            offset: 1.0,
            color: Color {
                red: 0,
                green: 0,
                blue: 0,
                alpha: 0,
            },
        },
    ]
}

fn points() -> Vec<Vec2> {
    vec![
        Vec2 { x: 0.5, y: 0.1 },
        Vec2 { x: 0.9, y: 0.9 },
        Vec2 { x: 0.1, y: 0.9 },
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

fn white() -> Color {
    Color {
        red: 255,
        green: 255,
        blue: 255,
        alpha: 255,
    }
}
