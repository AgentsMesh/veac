use crate::arguments::{Cli, Command, LanguagePackageCommand};

#[test]
fn package_arguments_select_api_inspection_and_local_search() {
    assert!(matches!(
        parse(&["veac", "package", "api", "components"]).command,
        Command::LanguagePackage {
            command: LanguagePackageCommand::Api { .. }
        }
    ));
    assert!(matches!(
        parse(&["veac", "package", "inspect", "components"]).command,
        Command::LanguagePackage {
            command: LanguagePackageCommand::Inspect { .. }
        }
    ));
    assert!(matches!(
        parse(&["veac", "package", "search", "store", "title"]).command,
        Command::LanguagePackage {
            command: LanguagePackageCommand::Search { query, .. }
        } if query == "title"
    ));
    assert!(Cli::try_parse_from(["veac", "package", "components"]).is_err());
}

fn parse(arguments: &[&str]) -> Cli {
    Cli::try_parse_from(arguments).unwrap()
}
