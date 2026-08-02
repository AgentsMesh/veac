use std::fs;

use veac_codegen::emitter::{CodegenErrorKind, CodegenErrors};
use veac_plan::canonical::{LutInterpolation, MaterialKind};
use veac_plan::ResolvedInputKind;

use super::support::{
    bindings_with_original, emit_video_command, file_identity, graded_project, resolved,
};

#[test]
fn cube_parser_accepts_bom_comments_titles_domains_and_finite_tables() {
    let cube = concat!(
        "\u{feff}TITLE \"film # one\" # trailing comment\n",
        "DOMAIN_MIN -1 -1 -1\n",
        "LUT_1D_SIZE 2 # compact table\n",
        "0 0 0\n",
        "1 1 1 # final row\n",
    );
    emit(MaterialKind::Lut1d, cube.as_bytes(), "cube").unwrap();
}

#[test]
fn malformed_cube_headers_and_tables_fail_closed() {
    let cases: &[(&str, MaterialKind, &[u8])] = &[
        ("invalid UTF-8", MaterialKind::Lut1d, b"LUT_1D_SIZE 2\n\xff"),
        (
            "unquoted title",
            MaterialKind::Lut1d,
            b"TITLE nope\nLUT_1D_SIZE 2\n0 0 0\n1 1 1\n",
        ),
        (
            "empty title",
            MaterialKind::Lut1d,
            b"TITLE \"\"\nLUT_1D_SIZE 2\n0 0 0\n1 1 1\n",
        ),
        (
            "duplicate title",
            MaterialKind::Lut1d,
            b"TITLE \"a\"\nTITLE \"b\"\nLUT_1D_SIZE 2\n0 0 0\n1 1 1\n",
        ),
        (
            "unknown directive",
            MaterialKind::Lut1d,
            b"LUT_1D_SIZE 2\nFOO 1\n0 0 0\n1 1 1\n",
        ),
        ("missing size", MaterialKind::Lut1d, b"DOMAIN_MIN 0 0 0\n"),
        (
            "data before size",
            MaterialKind::Lut1d,
            b"0 0 0\nLUT_1D_SIZE 2\n1 1 1\n",
        ),
        (
            "noninteger size",
            MaterialKind::Lut1d,
            b"LUT_1D_SIZE 2.0\n0 0 0\n1 1 1\n",
        ),
        (
            "extra size token",
            MaterialKind::Lut1d,
            b"LUT_1D_SIZE 2 3\n0 0 0\n1 1 1\n",
        ),
        ("small size", MaterialKind::Lut1d, b"LUT_1D_SIZE 1\n0 0 0\n"),
        (
            "duplicate size",
            MaterialKind::Lut1d,
            b"LUT_1D_SIZE 2\nLUT_1D_SIZE 2\n0 0 0\n1 1 1\n",
        ),
        (
            "short table",
            MaterialKind::Lut1d,
            b"LUT_1D_SIZE 2\n0 0 0\n",
        ),
        (
            "long table",
            MaterialKind::Lut1d,
            b"LUT_1D_SIZE 2\n0 0 0\n1 1 1\n1 1 1\n",
        ),
        (
            "short row",
            MaterialKind::Lut1d,
            b"LUT_1D_SIZE 2\n0 0\n1 1 1\n",
        ),
        (
            "long row",
            MaterialKind::Lut1d,
            b"LUT_1D_SIZE 2\n0 0 0 0\n1 1 1\n",
        ),
        (
            "nonfinite row",
            MaterialKind::Lut1d,
            b"LUT_1D_SIZE 2\nNaN 0 0\n1 1 1\n",
        ),
        (
            "bad domain arity",
            MaterialKind::Lut1d,
            b"DOMAIN_MIN 0 0\nLUT_1D_SIZE 2\n0 0 0\n1 1 1\n",
        ),
        (
            "duplicate domain",
            MaterialKind::Lut1d,
            b"DOMAIN_MAX 1 1 1\nDOMAIN_MAX 2 2 2\nLUT_1D_SIZE 2\n0 0 0\n1 1 1\n",
        ),
        (
            "inverted domain",
            MaterialKind::Lut1d,
            b"DOMAIN_MIN 1 0 0\nDOMAIN_MAX 1 1 1\nLUT_1D_SIZE 2\n0 0 0\n1 1 1\n",
        ),
        (
            "late directive",
            MaterialKind::Lut1d,
            b"LUT_1D_SIZE 2\n0 0 0\nDOMAIN_MAX 1 1 1\n1 1 1\n",
        ),
    ];
    for (name, kind, bytes) in cases {
        assert_code(emit(*kind, bytes, "cube"), "LUT_RESOURCE_INVALID", name);
    }
}

#[test]
fn declared_kind_and_dimension_budgets_are_enforced() {
    for (kind, bytes) in [
        (MaterialKind::Lut1d, b"LUT_3D_SIZE 2\n".as_slice()),
        (MaterialKind::Lut3d, b"LUT_1D_SIZE 2\n".as_slice()),
    ] {
        assert_code(emit(kind, bytes, "cube"), "LUT_RESOURCE_INVALID", "kind");
    }
    for (kind, bytes) in [
        (MaterialKind::Lut1d, b"LUT_1D_SIZE 65537\n".as_slice()),
        (MaterialKind::Lut3d, b"LUT_3D_SIZE 65\n".as_slice()),
    ] {
        assert_code(emit(kind, bytes, "cube"), "LUT_RESOURCE_LIMIT", "dimension");
    }
}

fn emit(kind: MaterialKind, bytes: &[u8], extension: &str) -> Result<(), CodegenErrors> {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join(format!("look.{extension}"));
    fs::write(&path, bytes).unwrap();
    let interpolation = if kind == MaterialKind::Lut1d {
        LutInterpolation::Linear
    } else {
        LutInterpolation::Tetrahedral
    };
    let mut project = graded_project(kind, interpolation);
    project.project.materials.last_mut().unwrap().identity = Some(file_identity(&path));
    let plan = resolved(&project);
    let resource = plan
        .inputs
        .iter()
        .find(|input| matches!(input.kind, ResolvedInputKind::Resource { .. }))
        .unwrap();
    let local = bindings_with_original(&plan, &resource.id, path);
    emit_video_command(&plan, &local).map(|_| ())
}

fn assert_code(result: Result<impl Sized, CodegenErrors>, code: &str, name: &str) {
    let error = match result {
        Ok(_) => panic!("{name} unexpectedly succeeded"),
        Err(error) => error,
    };
    assert_eq!(
        error.diagnostics()[0].kind,
        CodegenErrorKind::InvalidResourceBinding,
        "{name}"
    );
    assert_eq!(error.diagnostics()[0].code, code, "{name}: {error}");
}
