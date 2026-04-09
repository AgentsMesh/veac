/// Output the complete VEAC language syntax reference.
///
/// Embeds all language-reference documentation at compile time,
/// so AI agents can retrieve the full spec via `veac syntax`.

const SYNTAX_REFERENCE: &str = concat!(
    include_str!("../../../../docs/language-reference/README.md"),
    "\n\n---\n\n",
    include_str!("../../../../docs/language-reference/literals.md"),
    "\n\n---\n\n",
    include_str!("../../../../docs/language-reference/project.md"),
    "\n\n---\n\n",
    include_str!("../../../../docs/language-reference/assets.md"),
    "\n\n---\n\n",
    include_str!("../../../../docs/language-reference/variables-and-includes.md"),
    "\n\n---\n\n",
    include_str!("../../../../docs/language-reference/timeline-and-tracks.md"),
    "\n\n---\n\n",
    include_str!("../../../../docs/language-reference/clips.md"),
    "\n\n---\n\n",
    include_str!("../../../../docs/language-reference/transitions.md"),
    "\n\n---\n\n",
    include_str!("../../../../docs/language-reference/overlays.md"),
);

pub fn cmd_syntax() -> Result<(), Box<dyn std::error::Error>> {
    println!("{SYNTAX_REFERENCE}");
    Ok(())
}
