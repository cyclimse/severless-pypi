/// Reference: https://peps.python.org/pep-0691/
use serde::{Deserialize, Serialize};

pub const SUPPORTED_CONTENT_TYPE: &str = "application/vnd.pypi.simple.v1+json";

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct IndexProject {
    meta: IndexMeta,
    name: String,
    pub files: Vec<IndexFile>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct IndexMeta {
    api_version: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct IndexFile {
    pub filename: String,
    pub url: String,
}
