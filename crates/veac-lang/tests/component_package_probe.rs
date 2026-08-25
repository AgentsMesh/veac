use std::path::Path;
use veac_lang::program::{CompilerDatabase, FileSystemLoader};

#[test]
fn component_package_sources_compile() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib/veac-components");
    for entry in [
        "main.veac",
        "layout.veac",
        "text.veac",
        "motion.veac",
        "media.veac",
        "audio.veac",
        "delivery.veac",
        "components/card.veac",
    ] {
        let path = root.join(entry);
        let (loader, source) = FileSystemLoader::for_entry(&path).unwrap();
        let interface = CompilerDatabase::default().module_interface(source, &loader);
        assert!(interface.is_ok(), "{entry}: {:?}", interface.err());
    }
}
