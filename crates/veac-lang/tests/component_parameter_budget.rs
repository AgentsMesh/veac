use veac_lang::program::compile_source;

const PARAMETER_WIDTH: usize = 256;
const INSTANCE_COUNT: usize = 256;

#[test]
fn aggregate_parameter_work_accepts_exact_boundary_and_rejects_first_over() {
    compile_source(&source("")).unwrap();
    assert_code(
        &source("instance sequence overflow from single { bind only 1; }"),
        "PROGRAM_PARAMETER_BUDGET",
    );
}

#[test]
fn current_instance_semantics_precede_an_exhausted_aggregate_budget() {
    for (instance, code) in [
        (
            "instance sequence invalid from single {}",
            "PROGRAM_PARAMETER_MISSING",
        ),
        (
            "instance sequence invalid from single { bind only \"text\"; }",
            "PROGRAM_PARAMETER_TYPE",
        ),
        (
            "instance sequence invalid from single { bind only absent; }",
            "PROGRAM_PARAMETER_EXPRESSION",
        ),
        (
            "instance sequence invalid from single { bind unknown 1; }",
            "PROGRAM_PARAMETER_UNKNOWN",
        ),
    ] {
        assert_code(&source(instance), code);
    }
}

fn source(last_instance: &str) -> String {
    let parameters = (0..PARAMETER_WIDTH)
        .map(|index| format!("  param scalar p{index:03} default 1;"))
        .collect::<Vec<_>>()
        .join("\n");
    let instances = (0..INSTANCE_COUNT)
        .map(|index| format!("instance sequence item-{index:03} from wide {{}}"))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        r#"component sequence wide {{
{parameters}
  body {{}}
}}
component sequence single {{ param scalar only; body {{}} }}
{instances}
{last_instance}
project parameter-budget {{
  settings {{
    timebase 1/1000; canvas 1px by 1px;
    frame-rate 1fps; sample-rate 8000hz;
  }}
  entry sequence main; sequence main {{}}
}}"#
    )
}

fn assert_code(source: &str, expected: &str) {
    let error = compile_source(source).unwrap_err();
    assert_eq!(error.as_slice()[0].code, expected);
}
