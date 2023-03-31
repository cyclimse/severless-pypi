use serde::{Deserialize, Serialize};
use serde_plain::derive_display_from_serialize;

const TEST_PYPI_URL: &str = "test.pypi.org";

fn default_pypi_index() -> String {
    TEST_PYPI_URL.to_string()
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum ScalewayRegion {
    FrPar,
    NlAms,
    PlWaw,
}

derive_display_from_serialize!(ScalewayRegion);

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
