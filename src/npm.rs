use crate::Satisfied;
use crate::{Dependency, DependencyHolder};

use std::collections::HashMap;

use node_semver::{Range, Version};
use serde::{Deserialize, Serialize};

impl Satisfied for Range {
    fn is_satisfied_by(&self, version: &str) -> bool {
        let parsed: Version = version.parse().unwrap();
        self.satisfies(&parsed)
    }
}

pub type NpmDependency = Dependency<Range>;
pub type PackageJson = DependencyHolder<Range>;

#[derive(Serialize, Deserialize, Debug)]
pub struct PackageAuthor {
    name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PackageData {
    author: PackageAuthor,
    version: Version,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PackageJsonRaw {
    dependencies: HashMap<String, String>,
    dev_dependencies: HashMap<String, String>,
}

impl NpmDependency {
    /// Creates a new npm-compatible dependency from the given
    /// name and version string.
    ///
    /// ```
    /// # use depchk::npm::NpmDependency;
    /// # use depchk::Satisfied;
    ///
    /// let dependency = NpmDependency::new("axios", "^0.12");
    ///
    /// assert!(dependency.is_satisfied_by("0.12.0"));
    /// ```
    pub fn new(name: &str, version: &str) -> Self {
        NpmDependency::try_new(name, version).unwrap()
    }

    /// Attempts to create a new npm-compatible dependency from the
    /// given name and version string. However, if the version
    /// string is not parsable, returns None.
    ///
    /// ```
    /// # use depchk::npm::NpmDependency;
    ///
    /// let dependency = NpmDependency::try_new("axios", "^0.12");
    /// let invalid = NpmDependency::try_new("axios", ">=0.10,!=0.11,<0.13");
    ///
    /// assert!(dependency.is_some());
    /// assert!(invalid.is_none());
    /// ```
    pub fn try_new(name: &str, version: &str) -> Option<Self> {
        let parsed: Range = version.parse().ok()?;

        Some(NpmDependency::create(name, parsed))
    }

    /// Creates a vector of `Dependency` instances from a given hashmap.
    /// This is used to convert the `package.json` format (in which the `dependencies` and
    /// `devDependencies` keys are just a simple dictionary instead of an array).
    ///
    /// ```
    /// # use depchk::npm::NpmDependency;
    /// # use std::collections::HashMap;
    ///
    /// let map = HashMap::from([
    ///     ("axios".to_string(), "0.12".to_string())
    /// ]);
    ///
    /// let dependencies = NpmDependency::from_map(map);
    ///
    /// assert_eq!(dependencies.len(), 1);
    /// ```
    pub fn from_map(map: HashMap<String, String>) -> Vec<Self> {
        map.iter().map(|(k, v)| NpmDependency::new(k, v)).collect()
    }
}

impl From<PackageJsonRaw> for PackageJson {
    fn from(value: PackageJsonRaw) -> Self {
        PackageJson::new(
            NpmDependency::from_map(value.dependencies),
            NpmDependency::from_map(value.dev_dependencies),
        )
    }
}

impl From<String> for PackageJson {
    fn from(value: String) -> Self {
        let raw: PackageJsonRaw = serde_json::from_str(&value).unwrap();

        PackageJson::from(raw)
    }
}

impl From<&str> for PackageJson {
    fn from(file_name: &str) -> Self {
        let contents = std::fs::read_to_string(file_name).unwrap();

        PackageJson::from(contents)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_dependency_creates_successfully_with_raw_version() {
        let dependency = NpmDependency::new("axios", "0.12.0");

        assert_eq!(dependency.get_name(), "axios");
        assert!(dependency.is_satisfied_by("0.12.0"));
        assert!(!dependency.is_satisfied_by("0.12.1"));
    }

    #[test]
    fn package_dependency_creates_successfully_with_simple_requirements() {
        let dependency = NpmDependency::new("axios", "^0.12");

        assert_eq!(dependency.get_name(), "axios");
        assert!(dependency.is_satisfied_by("0.12.0"));
        assert!(dependency.is_satisfied_by("0.12.1"));
        assert!(!dependency.is_satisfied_by("0.13.0"));
    }

    #[test]
    fn package_dependency_creates_successfully_with_complex_requirements() {
        let dependency = NpmDependency::new("axios", "0.9 || >=0.11 <0.13");

        assert_eq!(dependency.get_name(), "axios");
        assert!(dependency.is_satisfied_by("0.9.0"));
        assert!(dependency.is_satisfied_by("0.11.0"));
        assert!(dependency.is_satisfied_by("0.12.0"));
        assert!(!dependency.is_satisfied_by("0.10.0"));
        assert!(!dependency.is_satisfied_by("0.13.0"));
    }
}
