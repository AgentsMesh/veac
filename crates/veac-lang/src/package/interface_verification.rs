use super::{PackageDiscovery, PackageError, PackageErrorKind, PackageIdentity};

pub(super) fn verify(discovery: &PackageDiscovery) -> Result<(), PackageError> {
    let loader = super::PackageSourceLoader::from_discovery(discovery)?;
    let database = crate::program::CompilerDatabase::default();
    for package in std::iter::once(&discovery.root).chain(&discovery.dependencies) {
        let entry = super::source_loader::relative_id(&package.path, &package.entry)?;
        let source = loader.load_package(&package.package, &entry)?;
        let interface = database
            .module_interface(source, &loader)
            .map_err(|diagnostics| {
                contract(format!(
                    "package {} entry is not a valid exported module interface: {diagnostics}",
                    label(&package.package)
                ))
            })?;
        let derived = super::api::from_module_interface(package.package.clone(), &interface);
        if derived != package.api {
            return Err(contract(format!(
                "package {} API metadata does not match its compiled module interface",
                label(&package.package)
            )));
        }
    }
    Ok(())
}

fn label(value: &PackageIdentity) -> String {
    format!("{}@{}", value.name, value.version)
}

fn contract(message: impl Into<String>) -> PackageError {
    PackageError::new(PackageErrorKind::Contract, message)
}
