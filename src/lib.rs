pub mod npm;

use std::error::Error;

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

type DirectResult<T> = Result<T, Box<dyn Error>>;
type OptionalResult<T> = DirectResult<Option<T>>;

pub type DependencyMismatchResult = DirectResult<VersionMismatch>;
type DependencyCheckResult = OptionalResult<VersionMismatch>;

#[async_trait]
pub trait Dependency {
    fn get_name(&self) -> &str;

    fn is_satisfied_by(&self, version: &str) -> bool;

    async fn check_version(&self, client: &Client) -> DependencyCheckResult;
}

pub trait DependencyFileParser {
    type Output: Dependency;

    fn parse_file(file_name: &str) -> Result<ProjectDependencies<Self::Output>, Box<dyn Error>>;
}

pub struct ProjectDependencies<T: Dependency> {
    dependencies: Vec<T>,
    dev_dependencies: Vec<T>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VersionMismatch {
    name: String,
    constraint: String,
    version: String,
}

#[derive(Serialize, Deserialize)]
pub struct Mismatches {
    pub dependencies: Vec<VersionMismatch>,
    pub dev_dependencies: Vec<VersionMismatch>,
}

impl Mismatches {
    pub fn concat(mut self) -> Vec<VersionMismatch> {
        let mut all = Vec::with_capacity(self.dependencies.len() + self.dev_dependencies.len());

        all.append(&mut self.dependencies);
        all.append(&mut self.dev_dependencies);

        all
    }
}

impl VersionMismatch {
    pub fn destruct(&self) -> (&str, &str, &str) {
        (&self.name, &self.constraint, &self.version)
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl<T: Dependency> ProjectDependencies<T> {
    fn new(deps: Vec<T>, dev_deps: Vec<T>) -> Self {
        ProjectDependencies {
            dependencies: deps,
            dev_dependencies: dev_deps,
        }
    }

    pub async fn check_dependencies(&self, client: &Client) -> Vec<DependencyMismatchResult> {
        check_dependencies(client, &self.dependencies).await
    }

    pub async fn check_dev_dependencies(&self, client: &Client) -> Vec<DependencyMismatchResult> {
        check_dependencies(client, &self.dev_dependencies).await
    }
}

pub async fn check_dependencies<T: Dependency>(
    client: &Client,
    dependencies: &[T],
) -> Vec<DependencyMismatchResult> {
    let mut handlers = Vec::with_capacity(dependencies.len());

    for dependency in dependencies {
        handlers.push(dependency.check_version(client).await);
    }

    let mut results = Vec::with_capacity(handlers.len());

    for result in handlers {
        if result.is_err() || result.as_ref().unwrap().is_some() {
            results.push(result.map(|mismatch| mismatch.unwrap()))
        }
    }

    results
}
