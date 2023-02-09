pub trait Satisfied {
    /// Provides a unified way to check that a given flavour of
    /// version requirement is satisfied by the string representation
    /// of a given version
    fn is_satisfied_by(&self, version: &str) -> bool;
}

pub struct Dependency<T: Satisfied> {
    name: String,
    version: T,
}

pub struct DependencyHolder<T: Satisfied> {
    dependencies: Vec<Dependency<T>>,
    dev_dependencies: Vec<Dependency<T>>,
}

impl<T: Satisfied> Dependency<T> {
    pub fn create(name: &str, version: T) -> Self {
        Dependency {
            name: name.to_string(),
            version,
        }
    }
    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_version(&self) -> &T {
        &self.version
    }
}

impl<T: Satisfied> Satisfied for Dependency<T> {
    fn is_satisfied_by(&self, version: &str) -> bool {
        self.version.is_satisfied_by(version)
    }
}

impl<T: Satisfied> DependencyHolder<T> {
    pub fn new(dependencies: Vec<Dependency<T>>, dev_dependencies: Vec<Dependency<T>>) -> Self {
        DependencyHolder {
            dependencies,
            dev_dependencies,
        }
    }
    /// Exposes a way to borrow a slice for dependencies
    pub fn dependencies(&self) -> &[Dependency<T>] {
        &self.dependencies
    }

    /// Exposes a way to borrow a slice for dev dependencies
    pub fn dev_dependencies(&self) -> &[Dependency<T>] {
        &self.dev_dependencies
    }
}
