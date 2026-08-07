use std::collections::BTreeSet;

use veac_ir::{
    ClipSource, Generator, Gradient, MaterialKind, MaterialSource, Paint, ProjectEnvelope,
    VectorGeometry,
};

use super::support;

#[test]
fn generator_claims_have_typed_ir_evidence() {
    support::assert_preview_evidence("generators.json", evidence);
}

fn evidence(project: &ProjectEnvelope) -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    for material in &project.project.materials {
        if material.kind == MaterialKind::Image {
            add(&mut found, "resource.kind.image");
        }
        if matches!(&material.source, MaterialSource::File { .. }) {
            add(&mut found, "resource.locator.file");
        }
    }
    for clip in support::clips(project) {
        match &clip.source {
            ClipSource::Generated { generator } => add_generator(generator, &mut found),
            ClipSource::Media { material_id }
                if project.project.materials.iter().any(|material| {
                    material.id == *material_id && material.kind == MaterialKind::Image
                }) =>
            {
                add(&mut found, "source.media.image");
            }
            _ => {}
        }
    }
    found
}

fn add_generator(value: &Generator, found: &mut BTreeSet<String>) {
    match value {
        Generator::Transparent => add(found, "generator.transparent"),
        Generator::Silence => add(found, "generator.silence"),
        Generator::Solid { .. } => add(found, "generator.solid"),
        Generator::Gradient { gradient } => add_gradient(gradient, found),
        Generator::Shape { shape } => {
            add_geometry(&shape.geometry, found);
            if let Some(paint) = &shape.fill {
                add_paint(paint, "generator.paint", found);
            }
            if let Some(stroke) = &shape.stroke {
                match &stroke.paint {
                    Paint::Solid { .. } => add(found, "generator.stroke.solid"),
                    Paint::Gradient { gradient } => {
                        add(found, "generator.stroke.gradient");
                        add_gradient(gradient, found);
                    }
                }
            }
        }
    }
}

fn add_gradient(value: &Gradient, found: &mut BTreeSet<String>) {
    let stops = match value {
        Gradient::Linear { stops, .. } => {
            add(found, "generator.gradient.linear");
            stops
        }
        Gradient::Radial { stops, .. } => {
            add(found, "generator.gradient.radial");
            stops
        }
    };
    add(found, "generator.gradient.custom-geometry");
    if stops.len() >= 3 {
        add(found, "generator.gradient.multi-stop");
    }
}

fn add_geometry(value: &VectorGeometry, found: &mut BTreeSet<String>) {
    let id = match value {
        VectorGeometry::Rectangle { .. } => "generator.shape.rectangle",
        VectorGeometry::RoundedRectangle { .. } => "generator.shape.rounded-rectangle",
        VectorGeometry::Ellipse { .. } => "generator.shape.ellipse",
        VectorGeometry::Polygon { .. } => "generator.shape.polygon",
        VectorGeometry::Path { .. } => "generator.shape.path",
    };
    add(found, id);
}

fn add_paint(value: &Paint, prefix: &str, found: &mut BTreeSet<String>) {
    add(found, prefix);
    match value {
        Paint::Solid { .. } => add(found, &format!("{prefix}.solid")),
        Paint::Gradient { gradient } => {
            add(found, &format!("{prefix}.gradient"));
            add_gradient(gradient, found);
        }
    }
}

fn add(found: &mut BTreeSet<String>, id: &str) {
    found.insert(id.to_owned());
}
