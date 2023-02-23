use serde::Deserialize;

const TEST_PYPI_URL: &str = "test.pypi.org";

fn default_pypi_index() -> String {
    TEST_PYPI_URL.to_string()
}

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub port: Option<i32>,
    /// Registry to target.
    /// Example: test.pypi.org
    #[serde(default = "default_pypi_index")]
    pub pypi_index: String,

    pub worker_url: String,
}
