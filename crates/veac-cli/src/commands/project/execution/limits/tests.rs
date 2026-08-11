use super::*;

#[test]
fn empty_manifest_uses_one_cpu_job() {
    let mut manifest: ProjectManifestV1 = serde_json::from_value(serde_json::json!({
        "schema":"veac.project","version":1,"id":"demo",
        "paths":{"source_base":"src","material_root":"materials","build_root":"build","cache_root":"cache","delivery_root":"dist"},
        "defaults":{"profile":null,"locale":null,"max_instances_per_target":1,"max_total_instances":1},
        "locales":[],"profiles":[],"targets":[]
    })).unwrap();
    manifest.profiles.clear();
    let limits = from_manifest(&manifest).unwrap();
    assert_eq!(limits.jobs, 1);
    assert_eq!(limits.resources, ResourceClaim::new(1, 0, 0));
}

#[test]
fn profile_policies_contribute_their_maximum_resources() {
    let manifest: ProjectManifestV1 = serde_json::from_value(serde_json::json!({
        "schema":"veac.project","version":1,"id":"demo",
        "paths":{"source_base":"src","material_root":"materials","build_root":"build","cache_root":"cache","delivery_root":"dist"},
        "defaults":{"profile":null,"locale":null,"max_instances_per_target":1,"max_total_instances":1},
        "locales":[],
        "profiles":[
            {"id":"serial","execution":{"kind":"serial"},"proxy":{"kind":"disabled"},"segmentation":{"kind":"whole"}},
            {"id":"parallel","execution":{"kind":"parallel","max_tasks":4},"proxy":{"kind":"disabled"},"segmentation":{"kind":"whole"}},
            {"id":"bounded","execution":{"kind":"resource_aware","max_tasks":3,"cpu_threads":8,"memory_mib":4096,"gpu_slots":2},"proxy":{"kind":"disabled"},"segmentation":{"kind":"whole"}}
        ],
        "targets":[]
    }))
    .unwrap();
    let limits = from_manifest(&manifest).unwrap();
    assert_eq!(limits.jobs, 4);
    assert_eq!(limits.resources, ResourceClaim::new(8, 4096, 2));
}
