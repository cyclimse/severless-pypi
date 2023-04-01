/// Reference: https://peps.python.org/pep-0691/
use serde::{Deserialize, Serialize};

pub const SUPPORTED_CONTENT_TYPE: &str = "application/vnd.pypi.simple.v1+json";

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct IndexProject {
    pub meta: IndexMeta,
    pub name: String,
    pub files: Vec<IndexFile>,
}

impl IndexProject {
    pub fn canonical_name(&self) -> String {
        self.name.replace('-', "_")
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct IndexMeta {
    pub api_version: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "kebab-case")]
pub struct IndexFile {
    pub filename: String,
    pub url: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum IndexFileKind {
    Tarball,
    Wheel,
    Unsupported,
}

impl IndexFile {
    /// Get kind
    pub fn kind(&self) -> IndexFileKind {
        let extension = self.filename.split('.').last();
        match extension {
            Some("gz") => IndexFileKind::Tarball,
            Some("whl") => IndexFileKind::Wheel,
            _ => IndexFileKind::Unsupported,
        }
    }

    /// Get the file version
    pub fn version(&'_ self) -> Option<&'_ str> {
        let middle = self.filename.split('-').nth(1)?;
        Some(middle.trim_end_matches(".tar.gz"))
    }

    /// Get wheel tags
    /// See: https://packaging.python.org/en/latest/specifications/platform-compatibility-tags/
    pub fn compatibility_tags(&'_ self) -> (&'_ str, &'_ str, &'_ str) {
        let v: Vec<_> = self.filename.split('-').skip(2).collect();
        match &v[..] {
            &[py, abi, pfm, ..] => (py, abi, pfm.split('.').next().unwrap_or_default()),
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_file_kind() {
        let wheel = IndexFile {
            filename: "scw_serverless-0.0.4-py3-none-any.whl".to_string(),
            ..Default::default()
        };

        assert_eq!(wheel.kind(), IndexFileKind::Wheel);

        let archive = IndexFile {
            filename: "scw_serverless-0.0.4.tar.gz".to_string(),
            ..Default::default()
        };

        assert_eq!(archive.kind(), IndexFileKind::Tarball);
    }

    #[test]
    fn test_index_file_version() {
        let wheel = IndexFile {
            filename: "scw_serverless-0.0.4-py3-none-any.whl".to_string(),
            ..Default::default()
        };

        assert_eq!(wheel.version(), Some("0.0.4"));

        let archive = IndexFile {
            filename: "scw_serverless-0.0.4.tar.gz".to_string(),
            ..Default::default()
        };

        assert_eq!(archive.version(), Some("0.0.4"));
    }
}
