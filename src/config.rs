use core::fmt;
use std::fmt::Display;

use serde::Deserialize;

const TEST_PYPI_URL: &str = "test.pypi.org";

fn default_pypi_index() -> String {
    TEST_PYPI_URL.to_string()
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum ScalewayRegion {
    FrPar,
    NlAms,
    PlWaw,
}

impl Display for ScalewayRegion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub port: Option<i32>,

    pub scw_access_key: String,
    pub scw_secret_key: String,
    pub scw_default_region: ScalewayRegion,

    pub s3_bucket: String,
    pub s3_endpoint: String,

    /// Registry to target.
    /// Example: test.pypi.org
    #[serde(default = "default_pypi_index")]
    pub pypi_index: String,

    pub worker_url: String,
}
