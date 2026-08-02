use super::*;

#[test]
fn relative_materialization_parents_bind_to_the_current_directory() {
    let mut guard = || true;
    let (parent, name) = checked(Path::new("output.bin"), &mut guard).unwrap();
    assert_eq!(parent, std::env::current_dir().unwrap());
    assert_eq!(name, "output.bin");
}
