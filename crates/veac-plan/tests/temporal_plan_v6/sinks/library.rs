use veac_plan::canonical::*;

use crate::fixtures::provenance;

pub(super) struct Bindings {
    pub scalar: TemporalBindingId,
    pub angle: TemporalBindingId,
    pub vector: TemporalBindingId,
    pub point: TemporalBindingId,
    pub rect: TemporalBindingId,
}

impl Bindings {
    pub(super) fn values(&self) -> Vec<TemporalBindingId> {
        vec![
            self.scalar.clone(),
            self.angle.clone(),
            self.vector.clone(),
            self.point.clone(),
            self.rect.clone(),
        ]
    }
}

pub(super) fn install(project: &mut ProjectEnvelope) -> Bindings {
    project.temporal.provenance = vec![provenance("sinks")];
    let scalar = bind(project, "scalar", TemporalValue::Scalar { value: 0.5 });
    let angle = bind(project, "angle", TemporalValue::Angle { degrees: 12.0 });
    let vector = bind(
        project,
        "vector",
        TemporalValue::Vec2 {
            value: Vec2 { x: 1.0, y: 1.0 },
        },
    );
    let point = bind(
        project,
        "point",
        TemporalValue::Point {
            value: point_value(),
        },
    );
    let rect = bind(
        project,
        "rect",
        TemporalValue::Rect {
            value: Rect {
                x: 0.0,
                y: 0.0,
                width: 1.0,
                height: 1.0,
            },
        },
    );
    Bindings {
        scalar,
        angle,
        vector,
        point,
        rect,
    }
}

fn bind(project: &mut ProjectEnvelope, suffix: &str, value: TemporalValue) -> TemporalBindingId {
    let value_type = value.value_type();
    let program_id = TemporalProgramId::new(format!("tpg_{suffix}")).unwrap();
    let provenance_id = project.temporal.provenance[0].id.clone();
    let mut program = TemporalProgram {
        id: program_id.clone(),
        opset_version: TEMPORAL_OPSET_VERSION,
        inputs: Vec::new(),
        result_type: value_type,
        nodes: vec![TemporalNode {
            id: TemporalNodeId::new(0),
            value_type,
            kind: TemporalNodeKind::Literal { value },
            provenance_id: Some(provenance_id.clone()),
        }],
        result: TemporalNodeId::new(0),
        content_sha256: String::new(),
        provenance_id: provenance_id.clone(),
    };
    program.content_sha256 = temporal_program_digest(&program).unwrap();
    project.temporal.programs.push(program);
    let id = TemporalBindingId::new(format!("tbd_{suffix}")).unwrap();
    project.temporal.bindings.push(TemporalBinding {
        id: id.clone(),
        program_id,
        result_type: value_type,
        clocks: Vec::new(),
        parameters: Vec::new(),
        provenance_id,
    });
    id
}

fn point_value() -> Point {
    let half = Length {
        value: 0.5,
        unit: LengthUnit::Normalized,
    };
    Point { x: half, y: half }
}
