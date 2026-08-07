use crate::arguments::{Cli, Command, SchemaContract};

#[test]
fn language_spec_arguments_select_vocabulary_and_schema_modes() {
    assert!(matches!(
        parse(&["veac", "language-spec"]).command,
        Command::LanguageSpec { schema: false }
    ));
    assert!(matches!(
        parse(&["veac", "language-spec", "--schema"]).command,
        Command::LanguageSpec { schema: true }
    ));
    assert!(matches!(
        parse(&["veac", "schema", "--contract", "language-spec"]).command,
        Command::Schema {
            contract: SchemaContract::LanguageSpec,
            ..
        }
    ));
}

fn parse(arguments: &[&str]) -> Cli {
    Cli::try_parse_from(arguments).unwrap()
}
