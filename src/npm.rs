use crate::{
    Dependency, DependencyCheckResult, DependencyFileParser, ProjectDependencies, VersionMismatch,
};

use std::error::Error;
use std::{collections::HashMap, fs};

use async_trait::async_trait;
use node_semver::{Range, Version};
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// A struct representing an npm package dependency from a
/// package.json file.
pub struct NpmDependency {
    version: Range,
    name: String,
    api_url: String,
}

pub type PackageJson = ProjectDependencies<NpmDependency>;

/// A struct to encapsulate part of the data
/// provided by the NPM api
#[derive(Serialize, Deserialize, Debug)]
pub struct PackageData {
    version: String,
}

/// A struct used to deserialize a package.json
/// file into a format that can be more easily
/// processed into the appropriate dependency.
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
    /// # use depchk::Dependency;
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

        Some(NpmDependency {
            name: name.to_string(),
            version: parsed,
            api_url: format!("https://registry.npmjs.org/{}/latest", name),
        })
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

#[async_trait]
impl Dependency for NpmDependency {
    async fn check_version(&self, client: &Client) -> DependencyCheckResult {
        let res = client.get(&self.api_url).send().await?;
        let package_data: PackageData = res.json().await?;

        if self.is_satisfied_by(&package_data.version) {
            return Ok(None);
        }

        Ok(Some(VersionMismatch {
            name: self.name.clone(),
            constraint: self.version.to_string(),
            version: package_data.version,
        }))
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn is_satisfied_by(&self, version: &str) -> bool {
        let parsed: Version = version.parse().unwrap();

        self.version.satisfies(&parsed)
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

impl DependencyFileParser for PackageJson {
    type Output = NpmDependency;

    fn parse_file(file_name: &str) -> Result<ProjectDependencies<Self::Output>, Box<dyn Error>> {
        let file = fs::read_to_string(file_name)?;

        let raw: PackageJsonRaw = serde_json::from_str(&file)?;

        Ok(PackageJson::from(raw))
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
