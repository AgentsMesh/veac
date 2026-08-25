use super::loader_support::MutableLoader;
use super::{CHAIN, LEAF, STABLE};
use veac_lang::program::{CompilerDatabase, LoadedSource};

#[test]
fn changed_leaf_misses_dependent_interface_but_reuses_stable_interface() {
    let loader = MutableLoader::new([("leaf.veac", LEAF)]);
    let timing = LoadedSource {
        id: "timing.veac".into(),
        source: CHAIN.into(),
    };
    let stable = LoadedSource {
        id: "stable.veac".into(),
        source: STABLE.into(),
    };
    let database = CompilerDatabase::default();

    database.module_interface(timing.clone(), &loader).unwrap();
    database.module_interface(stable.clone(), &loader).unwrap();
    let before = database.statistics();

    loader.replace("leaf.veac", "module { export fn value() -> time { 3s } }\n");
    database.module_interface(timing, &loader).unwrap();
    database.module_interface(stable, &loader).unwrap();
    let after = database.statistics();

    assert_eq!(after.interface_misses - before.interface_misses, 1);
    assert_eq!(after.interface_hits - before.interface_hits, 1);
    assert_eq!(after.interface_insertions - before.interface_insertions, 1);
    assert_eq!(after.pending_semantic_invalidations, 0);
}
