use super::{contract, PackageError, PackageIdentity, SourceRegistry};

impl SourceRegistry {
    pub(crate) fn qualified_source(
        &self,
        importer: &str,
        package: &PackageIdentity,
        requested: &str,
    ) -> Result<String, PackageError> {
        let importer_source = self.source(importer)?;
        let owner = self
            .packages
            .get(&importer_source.owner)
            .expect("every source owner is registered");
        if &importer_source.owner != package && !owner.dependencies.contains(package) {
            return Err(contract(
                "import crosses an undeclared package dependency edge",
            ));
        }
        self.package_source(package, requested)
    }
}
