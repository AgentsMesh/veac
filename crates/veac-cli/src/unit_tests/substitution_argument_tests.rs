use crate::arguments::{Command, SubstitutionPolicy};
use crate::Cli;

#[test]
fn render_substitution_policies_default_to_prefer_and_parse_all_modes() {
    let default = Cli::try_parse_from(["veac", "render", "project.json"]).unwrap();
    assert!(matches!(
        default.command,
        Command::Render {
            proxy_policy: SubstitutionPolicy::Prefer,
            render_segment_policy: SubstitutionPolicy::Prefer,
            ..
        }
    ));
    let explicit = Cli::try_parse_from([
        "veac",
        "render",
        "project.json",
        "--proxy-policy",
        "require",
        "--render-segment-policy",
        "original",
    ])
    .unwrap();
    assert!(matches!(
        explicit.command,
        Command::Render {
            proxy_policy: SubstitutionPolicy::Require,
            render_segment_policy: SubstitutionPolicy::Original,
            ..
        }
    ));
    assert!(Cli::try_parse_from([
        "veac",
        "render",
        "project.json",
        "--proxy-policy",
        "fallback-on-corruption",
    ])
    .is_err());
}
