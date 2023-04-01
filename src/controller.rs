use crate::pep_691::{IndexFile, IndexFileKind, IndexProject};

// Generic for convenience when doing "pip install --target ..."
// from non-alpine computer.
pub const GENERATED_COMPATIBILITY_TAGS: &str = "py3-none-any";

pub fn is_compatible_with_scaleway_python_functions(file: &IndexFile) -> bool {
    if file.kind() != IndexFileKind::Wheel {
        return true;
    }
    let (py, _, pfm) = file.compatibility_tags();
    py.starts_with("py") && (pfm == "any" || pfm.starts_with("musllinux"))
}

pub fn redirect_to_reserved(project: &mut IndexProject) {
    let mut versions = std::collections::HashMap::new();
    let mut compatible_wheels = Vec::with_capacity(project.files.len());

    for file in &project.files {
        match file.kind() {
            IndexFileKind::Tarball => {
                if let Some(version) = file.version() {
                    versions.insert(version.to_string(), file.url.clone());
                }
            }
            IndexFileKind::Wheel => {
                // If a wheel is already compatible, we do not redirect to the worker.
                // Useful for Python only libraries.
                if is_compatible_with_scaleway_python_functions(file) {
                    compatible_wheels.push(file.clone());
                }
            }
            _ => {}
        }
    }

    project.files.clear();
    project.files.append(&mut compatible_wheels);

    // Create "virtual wheels" that will be built
    // from the archive.
    for (version, archive_url) in &versions {
        let filename = format!(
            "{}-{}-{}.whl",
            project.canonical_name(),
            version,
            GENERATED_COMPATIBILITY_TAGS
        );
        let url = format!("download/{}?archive={}", filename, archive_url);

        let file = IndexFile { filename, url };
        project.files.push(file);
    }
}

#[cfg(test)]
mod tests {
    use crate::pep_691::IndexMeta;

    use super::*;

    #[test]
    fn test_redirect_to_reserved_compatible() {
        let wheel = IndexFile {
            filename: "scw_serverless-0.0.4-py3-none-any.whl".to_string(),
            ..Default::default()
        };
        let mut project = IndexProject {
            meta: IndexMeta {
                api_version: "1.1".to_owned(),
            },
            name: "scw-serverless".to_owned(),
            files: vec![wheel],
        };

        redirect_to_reserved(&mut project);

        assert_eq!(project.files.len(), 1)
    }
}
